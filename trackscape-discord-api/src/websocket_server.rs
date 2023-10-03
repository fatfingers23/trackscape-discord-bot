//! A multi-room chat server.

use actix_web::web::Data;
use log::info;
use std::sync::Mutex;
use std::{
    collections::{HashMap, HashSet},
    io,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::{ConnId, Msg, VerificationCode};

#[derive(Serialize, Deserialize)]
pub struct DiscordToClanChatMessage {
    pub sender: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub enum WebSocketMessageType {
    ToClanChat,
    FromClanChat,
}

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage<T> {
    pub message: T,
    pub message_type: WebSocketMessageType,
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
    ClanChatFromConnectedClient {
        msg: Msg,
        conn: ConnId,
        res_tx: oneshot::Sender<()>,
    },
    ClanChatToConnectedClients {
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
    clan_chat_channels: HashMap<VerificationCode, HashSet<ConnId>>,

    /// Command receiver.
    cmd_rx: mpsc::UnboundedReceiver<Command>,

    ///visitor count
    connected_chatters_count: Data<Mutex<usize>>,
}

impl ChatServer {
    pub fn new(connected_chatters_count: Data<Mutex<usize>>) -> (Self, ChatServerHandle) {
        // create empty server
        let rooms = HashMap::with_capacity(4);

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                clan_chat_channels: rooms,
                cmd_rx,
                connected_chatters_count,
            },
            ChatServerHandle { cmd_tx },
        )
    }

    /// Send message to users in a Clan Chat Channel.
    ///
    /// `skip` is used to prevent messages triggered by a connection also being received by it.
    async fn send_system_message(
        &self,
        verification_code: &str,
        skip: ConnId,
        msg: impl Into<String>,
    ) {
        if let Some(sessions) = self.clan_chat_channels.get(verification_code) {
            let msg = msg.into();
            for conn_id in sessions {
                if *conn_id != skip {
                    if let Some(tx) = self.sessions.get(conn_id) {
                        // errors if client disconnected abruptly and hasn't been timed-out yet
                        let _ = tx.send(msg.clone());
                    }
                }
            }
        }
    }

    /// Send message to users who are connected via websocket to a clan by the `verification_code`.
    async fn send_clan_chat(&self, verification_code: &str, json_msg: impl Into<String>) {
        if let Some(sessions) = self.clan_chat_channels.get(verification_code) {
            let msg = json_msg.into();
            for conn_id in sessions {
                if let Some(tx) = self.sessions.get(conn_id) {
                    // errors if client disconnected abruptly and hasn't been timed-out yet
                    let _ = tx.send(msg.clone()).expect("Failed to send message");
                }
            }
        }
    }

    /// Send message to all other users in current Clan Chat Channel
    ///
    /// `conn` is used to find current room and prevent messages sent by a connection also being
    /// received by it.
    pub async fn send_message(&self, conn: ConnId, msg: impl Into<String>) {
        if let Some(room) = self
            .clan_chat_channels
            .iter()
            .find_map(|(room, participants)| participants.contains(&conn).then_some(room))
        {
            self.send_system_message(room, conn, msg).await;
        };
    }

    /// Register new session and assign unique ID to this session
    async fn connect(
        &mut self,
        tx: mpsc::UnboundedSender<Msg>,
        verification_code: String,
    ) -> ConnId {
        info!("Someone joined");

        // register session with random connection ID
        let id = Uuid::new_v4();

        self.sessions.insert(id, tx);

        self.clan_chat_channels
            .entry(verification_code.clone())
            .or_default()
            .insert(id);

        *self.connected_chatters_count.lock().unwrap() += 1;

        info!(
            "Total Connected Clients now: {}",
            self.connected_chatters_count.lock().unwrap()
        );

        id
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect {
                    conn_tx,
                    res_tx,
                    verification_code,
                } => {
                    let conn_id = self.connect(conn_tx, verification_code).await;
                    let _ = res_tx.send(conn_id);
                }
                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }
                Command::ClanChatFromConnectedClient { conn, msg, res_tx } => {
                    self.send_message(conn, msg).await;
                    let _ = res_tx.send(());
                }
                Command::ClanChatToConnectedClients {
                    sender,
                    msg,
                    res_tx,
                    verification_code,
                } => {
                    let websocket_message = WebSocketMessage {
                        message_type: WebSocketMessageType::ToClanChat,
                        message: DiscordToClanChatMessage {
                            sender: sender.clone(),
                            message: msg.clone(),
                        },
                    };

                    self.send_clan_chat(
                        &verification_code,
                        serde_json::to_string(&websocket_message).unwrap(),
                    )
                    .await;

                    let _ = res_tx.send(());
                }
            }
        }

        Ok(())
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        println!("Someone disconnected");
        // remove sender
        if self.sessions.remove(&conn_id).is_some() {
            // remove session from all rooms
            for (_, sessions) in &mut self.clan_chat_channels {
                sessions.remove(&conn_id);
            }
        }
        *self.connected_chatters_count.lock().unwrap() -= 1;
        info!(
            "Total Connected Clients now: {}",
            self.connected_chatters_count.lock().unwrap()
        );
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

    pub async fn send_discord_message_to_clan_chat(
        &self,
        sender: String,
        msg: Msg,
        verification_code: String,
    ) {
        let (res_tx, _res_rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::ClanChatToConnectedClients {
                sender,
                msg,
                res_tx,
                verification_code,
            })
            .unwrap();
    }

    /// Broadcast message to current room.
    pub async fn send_message_to_connected_clan(
        &self,
        conn: ConnId,
        msg: impl Into<String> + Clone,
    ) {
        let (res_tx, res_rx) = oneshot::channel();
        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::ClanChatFromConnectedClient {
                msg: msg.into(),
                conn,
                res_tx,
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
