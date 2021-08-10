use anyhow::{Context, Result};
use pgnparse::parser::PgnInfo;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::{analysis::get_game_id, game::Game};

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub games: Vec<Game>,
}

impl Database {
    pub fn default() -> Self {
        Database { games: vec![] }
    }

    pub fn read(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path);
        if let Ok(data) = data {
            serde_yaml::from_str(&data).context("Reading database file contents")
        } else {
            Ok(Database::default())
        }
    }

    pub fn add_game(&mut self, game: Game) {
        self.games.push(game);
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(&self)?;
        fs::write(path, content).context("Writing database file contents")
    }

    pub fn game_exists(&self, pgn_info: &PgnInfo) -> bool {
        let game_id = get_game_id(&pgn_info);
        self.games.iter().find(|game| game.id == game_id).is_none()
    }
}
