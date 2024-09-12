use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    vec,
};

// use rand::{thread_rng, Rng};
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;

pub enum RoomType {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub choice: String,
}

impl Player {
    pub fn new() -> Player {
        Player {
            id: Uuid::new_v4(),
            choice: String::new(),
        }
    }
}

pub struct Room {
    id: Uuid,
    players: Mutex<Vec<Player>>, // prob should use a fixed array of size 2
    room_type: RoomType,
    pub tx: Sender<String>,
}

impl Room {
    pub fn new(room_type: RoomType) -> Room {
        let (tx, _rx) = broadcast::channel(100);
        Room {
            id: Uuid::new_v4(),
            players: Mutex::new(vec![]),
            room_type,
            tx,
        }
    }
}

pub struct AppState {
    players: Mutex<HashMap<i32, Player>>,
    // pub tx: Sender<String>,
    num_of_players: Mutex<i32>,
    rooms: Mutex<Vec<Room>>,
}

impl AppState {
    // a lot of these should probably be bound to the their respective structs
    // for right now they will be here xD
    pub fn add_player_to_existing_room(&self, room_id: Uuid, player: Player) -> Uuid {
        let rooms = self.rooms.lock().unwrap();
        if let Some(found_room) = rooms.iter().find(|room| room.id == room_id) {
            let mut players = found_room.players.lock().unwrap();
            players.push(player);
        };
        room_id
    }

    pub fn add_player_to_new_room(&self, room_type: RoomType, player: Player) -> Uuid {
        let mut rooms = self.rooms.lock().unwrap();
        let room = Room::new(room_type);
        let room_id = room.id;

        {
            let mut players = room.players.lock().unwrap();
            players.push(player);
        } // apparently this implicitly calls the unlock of the mutex which removes the ownership of the room so I can push the room to rooms. :mind-blown:
        rooms.push(room);
        room_id
    }

    pub fn get_room_sender(&self, room_id: Uuid) -> Result<Sender<String>, &'static str> {
        let rooms = self.rooms.lock().unwrap();
        if let Some(found_room) = rooms.iter().find(|room| room.id == room_id) {
            return Ok(found_room.tx.clone());
        } else {
            return Err("could not get receiver for room: {room_id}");
        };
    }

    pub fn available_rooms(&self) -> Option<Uuid> {
        let rooms = self.rooms.lock().unwrap();
        let mut found_room: Option<Uuid> = None;

        if rooms.is_empty() {
            return None;
        }

        for room in rooms.iter() {
            let players = room.players.lock().unwrap();

            if players.len() < 2 {
                found_room = Some(room.id);
            }
        }

        found_room
    }

    // pub fn register_player(&self, player_id: i32) {
    //     let mut players = self.players.lock().unwrap();
    //     let mut player_count = self.num_of_players.lock().unwrap();
    //     match players.insert(
    //         player_id,
    //         Player {
    //             id: player_id,
    //             choice: String::new(),
    //         },
    //     ) {
    //         None => {
    //             println!("registered new player id: {player_id}");
    //             *player_count += 1;
    //         }
    //         Some(_) => (),
    //     }
    // }

    pub fn remove_player(&self, player_id: i32) {
        let mut players = self.players.lock().unwrap();
        match players.remove(&player_id) {
            Some(player) => {
                println!("player {} removed successfully", player.id);
                println!("remaining players: {:?}", players)
            }
            None => (),
        }
    }

    pub fn set_player_choice(&self, room_id: Uuid, player_id: Uuid, choice: String) {
        let rooms = self.rooms.lock().unwrap();

        if let Some(found_room) = rooms.iter().find(|room| room.id == room_id) {
            let mut players = found_room.players.lock().unwrap();
            if let Some(found_player) = players.iter_mut().find(|player| player.id == player_id) {
                found_player.choice = choice;
            };
        };

        // let mut players = self.players.lock().unwrap();
        // let player = match players.get_mut(&player_id) {
        //     Some(player) => player,
        //     None => todo!(),
        // };
        // player.choice = choice
    }

    pub fn has_player_chosen(&self, room_id: Uuid, player_id: Uuid) -> bool {
        let rooms = self.rooms.lock().unwrap();

        if let Some(found_room) = rooms.iter().find(|room| room.id == room_id) {
            let players = found_room.players.lock().unwrap();
            if let Some(found_player) = players.iter().find(|player| player.id == player_id) {
                return found_player.choice == "";
            };
        };
        return false;

        // let mut players = self.players.lock().unwrap();
        // let player = match players.get_mut(&player_id) {
        //     Some(player) => player,
        //     None => todo!(),
        // };
        // player.choice == ""
    }

    pub fn players_finished(&self, room_id: Uuid) -> bool {
        let rooms = self.rooms.lock().unwrap();

        if let Some(found_room) = rooms.iter().find(|room| room.id == room_id) {
            let players = found_room.players.lock().unwrap();
            if players.len() < 2 {
                return false;
            }
            return players.iter().all(|player| player.choice != "");
        };
        return false;

        // let players = self.players.lock().unwrap();
        // if players.keys().len() <= 1 {
        //     return false;
        // }
        // players.iter().all(|(_id, player)| player.choice != "")
    }

    pub fn calculate_winner(&self, room_id: Uuid) -> Option<Player> {
        let rooms = self.rooms.lock().unwrap();

        if let Some(found_room) = rooms.iter().find(|room| room.id == room_id) {
            let players = found_room.players.lock().unwrap();
            // if let Some(found_player) = players.iter().find(|player| player.id == player_id) {
            //     return found_player.choice == "";
            // };
            // let p: Vec<&Player> = players.iter().map(|(_, player)| player).collect();

            return calculate_rps(&players[0], &players[1]);
        };
        // // this is the worst function I have ever written. Don't judge
        // let players = self.players.lock().unwrap();

        // let p: Vec<&Player> = players.iter().map(|(_, player)| player).collect();

        // calculate_rps(p[0], p[1])
        return None;
    }

    pub fn new() -> Arc<AppState> {
        // let (tx, _rx) = broadcast::channel(100);
        Arc::new(AppState {
            // tx,
            players: Mutex::new(HashMap::new()),
            num_of_players: Mutex::new(0),
            rooms: Mutex::new(vec![]),
        })
    }
}

fn calculate_rps<'a>(p1: &'a Player, p2: &'a Player) -> Option<Player> {
    if p1.choice == "rock" && p2.choice == "scissor" {
        return Some(p1.clone());
    } else if p1.choice == "rock" && p2.choice == "paper" {
        Some(p2.clone())
    } else if p2.choice == "rock" && p1.choice == "paper" {
        Some(p1.clone())
    } else if p2.choice == "rock" && p1.choice == "scissors" {
        Some(p2.clone())
    } else {
        // for a draw
        return None;
    }
}
