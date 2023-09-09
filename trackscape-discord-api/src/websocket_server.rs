//! A multi-room chat server.

use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use rand::{thread_rng, Rng as _};
use serde::{Deserialize, Serialize};
use shuttle_runtime::tracing::{info, log};
use tokio::sync::{mpsc, oneshot};

use crate::{ConnId, Msg, RoomId};

#[derive(Serialize, Deserialize)]
pub struct DiscordToClanChatMessage {
    pub sender: String,
    pub message: String,
}

/// A command received by the [`ChatServer`].
#[derive(Debug)]
enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
        verification_code: String,
    },

    Disconnect {
        conn: ConnId,
    },

    List {
        res_tx: oneshot::Sender<Vec<RoomId>>,
    },

    Join {
        conn: ConnId,
        room: RoomId,
        res_tx: oneshot::Sender<()>,
    },

    Message {
        msg: Msg,
        conn: ConnId,
        res_tx: oneshot::Sender<()>,
        verification_code: String,
    },
    ClanChat {
        sender: String,
        msg: Msg,
        res_tx: oneshot::Sender<()>,
        verification_code: String,
    },
}

/// A multi-room chat server.
///
/// Contains the logic of how connections chat with each other plus room management.
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct ChatServer {
    /// Map of connection IDs to their message receivers.
    pub sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,

    /// Map of room name to participant IDs in that room.
    rooms: HashMap<RoomId, HashSet<ConnId>>,

    /// Tracks total number of historical connections established.
    visitor_count: Arc<AtomicUsize>,

    /// Command receiver.
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl ChatServer {
    pub fn new() -> (Self, ChatServerHandle) {
        // create empty server
        let mut rooms = HashMap::with_capacity(4);

        // create default room
        rooms.insert("main".to_owned(), HashSet::new());

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                rooms,
                visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            ChatServerHandle { cmd_tx },
        )
    }

    /// Send message to users in a room.
    ///
    /// `skip` is used to prevent messages triggered by a connection also being received by it.
    async fn send_system_message(&self, room: &str, skip: ConnId, msg: impl Into<String>) {
        if let Some(sessions) = self.rooms.get(room) {
            let msg = msg.into();

            for conn_id in sessions {
                // if *conn_id != skip {
                if let Some(tx) = self.sessions.get(conn_id) {
                    // errors if client disconnected abruptly and hasn't been timed-out yet
                    let _ = tx.send(msg.clone());
                }
                // }
            }
        }
    }

    /// Send message to users connected to a clan via the `verification_code`.
    async fn send_clan_chat(&self, verification_code: &str, json_msg: impl Into<String>) {
        if let Some(sessions) = self.rooms.get(verification_code) {
            let msg = json_msg.into();
            for conn_id in sessions {
                if let Some(tx) = self.sessions.get(conn_id) {
                    // errors if client disconnected abruptly and hasn't been timed-out yet
                    let _ = tx.send(msg.clone()).expect("Failed to send message");
                }
            }
        }
    }

    /// Send message to all other users in current room.
    ///
    /// `conn` is used to find current room and prevent messages sent by a connection also being
    /// received by it.
    pub async fn send_message(&self, conn: ConnId, msg: impl Into<String>) {
        if let Some(room) = self
            .rooms
            .iter()
            .find_map(|(room, participants)| participants.contains(&conn).then_some(room))
        {
            info!("Found a room {}", room);
            self.send_system_message(room, conn, msg).await;
        };
        info!("No room found");
    }

    /// Register new session and assign unique ID to this session
    async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        log::info!("Someone joined");

        // notify all users in same room
        self.send_system_message("main", 0, "Someone joined").await;

        // register session with random connection ID
        let id = thread_rng().gen::<usize>();
        // let id = 13025004057888928349;
        self.sessions.insert(id, tx);

        // auto join session to main room
        self.rooms.entry("main".to_owned()).or_default().insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_system_message("main", 0, format!("Total visitors {count}"))
            .await;

        // send id back
        id
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove sender
        if self.sessions.remove(&conn_id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&conn_id) {
                    rooms.push(name.to_owned());
                }
            }
        }

        // send message to other users
        for room in rooms {
            self.send_system_message(&room, 0, "Someone disconnected")
                .await;
        }
    }

    /// Returns list of created room names.
    fn list_rooms(&mut self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }

    /// Join room, send disconnect message to old room send join message to new room.
    async fn join_room(&mut self, conn_id: ConnId, room: String) {
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&conn_id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_system_message(&room, 0, "Someone disconnected")
                .await;
        }

        self.rooms.entry(room.clone()).or_default().insert(conn_id);

        self.send_system_message(&room, conn_id, "Someone connected")
            .await;
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            println!("Got command: {:?}", cmd);
            match cmd {
                Command::Connect {
                    conn_tx,
                    res_tx,
                    verification_code,
                } => {
                    let conn_id = self.connect(conn_tx).await;
                    self.join_room(conn_id, verification_code).await;
                    let _ = res_tx.send(conn_id);
                }

                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }

                Command::List { res_tx } => {
                    let _ = res_tx.send(self.list_rooms());
                }

                Command::Join { conn, room, res_tx } => {
                    self.join_room(conn, room).await;
                    let _ = res_tx.send(());
                }

                Command::Message {
                    conn,
                    msg,
                    res_tx,
                    verification_code,
                } => {
                    info!("attempting to Sending message to: {}", conn);

                    self.send_system_message(&verification_code, conn, msg.clone())
                        .await;
                    self.send_message(conn, msg).await;
                    let _ = res_tx.send(());
                }
                Command::ClanChat {
                    sender,
                    msg,
                    res_tx,
                    verification_code,
                } => {
                    info!("attempting to Sending message to: {}", verification_code);
                    let discord_to_clan_chat_message = DiscordToClanChatMessage {
                        sender: sender.clone(),
                        message: msg.clone(),
                    };
                    self.send_clan_chat(
                        &verification_code,
                        serde_json::to_string(&discord_to_clan_chat_message).unwrap(),
                    )
                    .await;
                    // self.send_system_message(&verification_code, 0, msg.clone())
                    //     .await;
                    // self.send_message(0, msg).await;
                    let _ = res_tx.send(());
                }
            }
        }

        Ok(())
    }
}

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ChatServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ChatServerHandle {
    /// Register client message sender and obtain connection ID.
    pub async fn connect(
        &self,
        conn_tx: mpsc::UnboundedSender<String>,
        verification_code: String,
    ) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Connect {
                conn_tx,
                res_tx,
                verification_code,
            })
            .unwrap();

        // unwrap: chat server does not drop out response channel
        res_rx.await.unwrap()
    }

    /// List all created rooms.
    pub async fn list_rooms(&self) -> Vec<String> {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: chat server should not have been dropped
        self.cmd_tx.send(Command::List { res_tx }).unwrap();

        // unwrap: chat server does not drop our response channel
        res_rx.await.unwrap()
    }

    /// Join `room`, creating it if it does not exist.
    pub async fn join_room(&self, conn: ConnId, room: impl Into<String>) {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Join {
                conn,
                room: room.into(),
                res_tx,
            })
            .unwrap();

        // unwrap: chat server does not drop our response channel
        res_rx.await.unwrap();
    }

    pub async fn send_discord_message_to_clan_chat(
        &self,
        sender: String,
        msg: Msg,
        verification_code: String,
    ) {
        let (res_tx, res_rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::ClanChat {
                sender,
                msg,
                res_tx,
                verification_code,
            })
            .unwrap();
    }

    /// Broadcast message to current room.
    pub async fn send_message(
        &self,
        conn: ConnId,
        msg: impl Into<String> + Clone,
        verification_code: String,
    ) {
        let (res_tx, res_rx) = oneshot::channel();
        info!("Sending message to: {}", conn);
        let msg_clone = msg.clone();
        info!("Message: {:?}", msg_clone.into());
        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                conn,
                res_tx,
                verification_code,
            })
            .unwrap();

        // unwrap: chat server does not drop our response channel
        res_rx.await.unwrap();
    }

    /// Unregister message sender and broadcast disconnection message to current room.
    pub fn disconnect(&self, conn: ConnId) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}
