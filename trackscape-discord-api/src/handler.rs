use std::time::{Duration, Instant};

use actix_ws::Message;
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use serde_json::Value;
use shuttle_runtime::tracing::{debug, error, log};
use tokio::{pin, sync::mpsc, time::interval};
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::ClanMessage;

use crate::websocket_server::WebSocketMessage;
use crate::websocket_server::WebSocketMessageType::FromClanChat;
use crate::{ChatServerHandle, ConnId};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

/// This will handle messages being sent from the websocket
pub async fn chat_ws(
    chat_server: ChatServerHandle,
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    verification_code: String,
) {
    log::info!("connected");

    let mut name = None;
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    // unwrap: chat server is not dropped before the HTTP server
    let conn_id = chat_server.connect(conn_tx, verification_code).await;

    let close_reason = loop {
        // most of the futures we process need to be stack-pinned to work with select()

        let tick = interval.tick();
        pin!(tick);

        let msg_rx = conn_rx.recv();
        pin!(msg_rx);

        // TODO: nested select is pretty gross for readability on the match
        let messages = select(msg_stream.next(), msg_rx);
        pin!(messages);

        match select(messages, tick).await {
            // commands & messages received from client
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => match msg {
                Message::Ping(bytes) => {
                    last_heartbeat = Instant::now();
                    session.pong(&bytes).await.expect("failed to send pong");
                }

                Message::Pong(_) => {
                    last_heartbeat = Instant::now();
                }

                Message::Text(text) => {
                    process_text_msg(&chat_server, &mut session, &text, conn_id, &mut name).await;
                }

                Message::Binary(_bin) => {
                    log::warn!("unexpected binary message");
                }

                Message::Close(reason) => break reason,

                _ => {
                    break None;
                }
            },

            // client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                log::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // chat messages received from other room participants
            Either::Left((Either::Right((Some(chat_msg), _)), _)) => {
                //Todo possibly see about a queue for messages. This can get overloaded in testing but takes thousands of messages at once
                session
                    .text(chat_msg)
                    .await
                    .expect("failed to send message to client");
            }

            // all connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "all connection message senders were dropped; chat server may have panicked"
            ),

            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    chat_server.disconnect(conn_id);

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

/// Process websocket messages that are to be sent to the server.
async fn process_text_msg(
    chat_server: &ChatServerHandle,
    _session: &mut actix_ws::Session,
    text: &str,
    conn: ConnId,
    _name: &mut Option<String>,
) {
    // strip leading and trailing whitespace (spaces, newlines, etc.)
    let msg = text.trim();

    let request_from_client: Result<Value, _> = serde_json::from_str(msg);
    if request_from_client.is_err() {
        error!("error parsing json");
        return;
    }
    let check_for_type = &request_from_client.unwrap()["message_type"];
    if check_for_type.is_null() {
        return;
    }

    if let Some(message_type) = check_for_type.as_str() {
        match message_type {
            "ToClanChat" => {
                let websocket_message: Result<WebSocketMessage<ClanMessage>, _> =
                    serde_json::from_str(msg);
                match websocket_message {
                    Ok(message) => {
                        let message_to_send = WebSocketMessage {
                            message_type: FromClanChat,
                            message: message.message,
                        };
                        let json_message = serde_json::to_string(&message_to_send).unwrap();
                        chat_server
                            .send_message_to_connected_clan(conn, json_message)
                            .await;
                    }
                    Err(_) => {}
                }
                return;
            }
            _ => {
                debug!("Unknown message type");
                return;
            }
        }
    }
}
