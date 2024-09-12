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
use state::{AppState, Player, RoomType};
use tokio::sync::mpsc;
use uuid::Uuid;

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
    let mut match_type = String::new();
    let (mut sink, mut stream) = stream.split();
    let (sender, mut reciever) = mpsc::channel::<String>(100);

    // gets match type from client
    loop {
        let message = match stream.next().await {
            Some(message) => message,
            None => break,
        };

        let message = match message {
            Ok(message) => message,
            Err(_) => todo!(),
        };

        if let Message::Text(message) = message {
            match_type = message;
            if match_type == "quick" || match_type == "private" {
                let _ = sink
                    .send(Message::Text(format!(
                        "Connected successfully. Finding opponent..."
                    )))
                    .await;
                break;
            }
        }
    }
    // prob should use a logger
    println!("requested match type: {match_type}");

    /*
    if quick play
        are there any rooms
            if no - create room - player goes in there - exit
            if yes
                search for empty slots in rooms
                    if no - create room - player goes in there - exit
                    if yes
                        player goes in there - proceed to start match
    if private match
        create - create room - player goes in there - exit
        join - find room with uuid - player goes in there - proceed to start match

    */
    let player = Player::new();
    let player_id = player.id;
    let room_id: Uuid;

    // if match_type == "quick" {
    if let Some(id) = app_state.available_rooms() {
        room_id = id;
        app_state.add_player_to_existing_room(id, player);
    } else {
        room_id = app_state.add_player_to_new_room(RoomType::Public, player);
    }
    println!("{room_id}");
    // }

    // if match_type == "private" {}

    /* everything below needs an update */

    // let mut rx = app_state.tx.subscribe();
    let room_tx = match app_state.get_room_sender(room_id) {
        Ok(rx) => rx,
        Err(e) => {
            println!("{e}");
            panic!()
        }
    };

    let mut room_rx = room_tx.subscribe();

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
            let message = match room_rx.recv().await {
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
                    // app_state.remove_player(player_id);
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
            let has_chosen = !app_state.has_player_chosen(room_id, player_id);
            if is_valid && !has_chosen {
                app_state.set_player_choice(room_id, player_id, message);

                // send to broadcast
                // let _ = app_state
                //     .tx
                //     .send(format!("player: {player_id} has made their choice"));
                let _ = room_tx.send(format!("player: {player_id} has made their choice"));
            }

            println!("{}", app_state.players_finished(room_id));
            if app_state.players_finished(room_id) {
                match app_state.calculate_winner(room_id) {
                    Some(winner) => {
                        // should be room tx not app state
                        let _ = room_tx.send(format!(
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
