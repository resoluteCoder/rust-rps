use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rand::{thread_rng, Rng};
use tokio::sync::broadcast::{self, Sender};

#[derive(Debug, Clone)]
pub struct Player {
    pub id: i32,
    pub choice: String,
}

pub struct AppState {
    players: Mutex<HashMap<i32, Player>>,
    pub tx: Sender<String>,
    num_of_players: Mutex<i32>,
}

impl AppState {
    pub fn register_player(&self, player_id: i32) {
        let mut players = self.players.lock().unwrap();
        let mut player_count = self.num_of_players.lock().unwrap();
        match players.insert(
            player_id,
            Player {
                id: player_id,
                choice: String::new(),
            },
        ) {
            None => {
                println!("registered new player id: {player_id}");
                *player_count += 1;
            }
            Some(_) => (),
        }
    }

    pub fn gen_player_id(&self) -> i32 {
        let mut rng = thread_rng();
        let player_set = self.players.lock().unwrap();
        let rand_id = rng.gen_range(1..=100);

        loop {
            let rand_id = rng.gen_range(1..=100);
            if !player_set.contains_key(&rand_id) {
                break;
            }
        }
        rand_id
    }

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

    pub fn set_player_choice(&self, player_id: i32, choice: String) {
        let mut players = self.players.lock().unwrap();
        let player = match players.get_mut(&player_id) {
            Some(player) => player,
            None => todo!(),
        };
        player.choice = choice
    }

    pub fn has_player_chosen(&self, player_id: i32) -> bool {
        let mut players = self.players.lock().unwrap();
        let player = match players.get_mut(&player_id) {
            Some(player) => player,
            None => todo!(),
        };
        player.choice == ""
    }

    pub fn players_finished(&self) -> bool {
        let players = self.players.lock().unwrap();
        if players.keys().len() <= 1 {
            return false;
        }
        players.iter().all(|(_id, player)| player.choice != "")
    }

    pub fn calculate_winner(&self) -> Option<Player> {
        // this is the worst function I have ever written. Don't judge
        let players = self.players.lock().unwrap();

        let p: Vec<&Player> = players.iter().map(|(_, player)| player).collect();

        calculate_rps(p[0], p[1])
    }

    pub fn new() -> Arc<AppState> {
        let (tx, _rx) = broadcast::channel(100);
        Arc::new(AppState {
            tx,
            players: Mutex::new(HashMap::new()),
            num_of_players: Mutex::new(0),
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
