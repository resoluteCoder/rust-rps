use std::sync::Arc;

mod state;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use state::AppState;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let app_state = AppState::new();

    let app = Router::new()
        .route("/", get(index))
        .route("/websocket", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, app_state))
}

async fn websocket(stream: WebSocket, app_state: Arc<AppState>) {
    let (mut sink, mut stream) = stream.split();
    let (sender, mut reciever) = mpsc::channel::<String>(16);

    let player_id = app_state.gen_player_id();
    app_state.register_player(player_id);

    let mut rx = app_state.tx.subscribe();

    // when connected immediately send you are connected as player id #
    let _ = sink
        .send(Message::Text(format!(
            "you have been connected as player: {player_id}"
        )))
        .await;

    // create a task to forward messages from mpsc to the sink
    tokio::spawn(async move {
        loop {
            let message = match reciever.recv().await {
                Some(message) => message,
                None => {
                    println!("could not forward message");
                    break;
                }
            };
            let _ = sink.send(Message::Text(message.into())).await;
        }
    });

    // create a broadcast task
    let broadcast_task_sender = sender.clone();
    // let tx = app_state.tx.clone();
    let mut broadcast = tokio::spawn(async move {
        loop {
            let message = match rx.recv().await {
                Ok(message) => message,
                Err(_) => todo!(),
            };

            let _ = broadcast_task_sender.send(message).await;
        }
    });

    let mut reader_task = tokio::spawn(async move {
        loop {
            let message = match stream.next().await {
                Some(message) => message,
                None => {
                    app_state.remove_player(player_id);
                    break;
                }
            };

            let message = match message {
                Ok(message) => message,
                Err(_) => todo!(),
            };

            let message = match message.into_text() {
                Ok(message) => message,
                Err(_) => todo!(),
            };

            let valid_choices = vec!["rock", "paper", "scissors"];
            let is_valid = valid_choices.contains(&message.as_str());
            let has_chosen = !app_state.has_player_chosen(player_id);
            if is_valid && !has_chosen {
                app_state.set_player_choice(player_id, message);

                // send to broadcast
                let _ = app_state
                    .tx
                    .send(format!("player: {player_id} has made their choice"));
            }

            println!("{}", app_state.players_finished());
            if app_state.players_finished() {
                match app_state.calculate_winner() {
                    Some(winner) => {
                        let _ = app_state.tx.send(format!(
                            "player: {} is the winner using {}!",
                            winner.id, winner.choice
                        ));
                    }
                    None => println!("inside player finished if block"),
                };
            }
        }
    });

    tokio::select! {
        _ = (&mut broadcast) => reader_task.abort(),
        _ = (&mut reader_task) => broadcast.abort(),
    }
    println!("shutting down single connection");
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../index.html"))
}
