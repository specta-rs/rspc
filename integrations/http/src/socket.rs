//! Websocket types by framework:
//!  Axum (tokio_tungstenite) - https://docs.rs/axum/latest/axum/extract/ws/struct.WebSocket.html
//!  Actix Web                - https://docs.rs/actix-ws/latest/actix_ws/
//!  Poem (tokio_tungstenite) - https://docs.rs/poem/latest/poem/web/websocket/struct.WebSocket.html
//!  Warp (tokio_tungstenite) - https://docs.rs/warp/latest/warp/filters/ws/struct.WebSocket.html
//!  Tide (tokio_tungstenite) - https://docs.rs/tide-websockets/latest/tide_websockets/
//!  Rocket (tokio_tungstenite) - https://docs.rs/rocket_ws/latest/rocket_ws/struct.WebSocket.html
//!  Hyper                    - Just use whatever.

// TODO: Copy this from `rspc_tauri`
struct WebsocketMsg {
    // Value
    // Done,
}

pub fn socket() {}
