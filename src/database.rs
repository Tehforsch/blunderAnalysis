use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

use crate::{game::Game, game_info::GameInfo};

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub games: Vec<Game>,
    pub path: PathBuf,
}

impl Database {
    pub fn empty(path: &Path) -> Self {
        Database { games: vec![], path: path.to_owned() }
    }

    pub fn read(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path);
        if let Ok(data) = data {
            let games = serde_yaml::from_str(&data).context("Reading database file contents")?;
            Ok(Database {
                games,
                path: path.to_owned(),
            })
        } else {
            Ok(Database::empty(path))
        }
    }

    pub fn add_game(&mut self, game: Game) {
        self.games.push(game);
    }

    pub fn write(&self) -> Result<()> {
        let content = serde_yaml::to_string(&self)?;
        fs::write(&self.path, content).context("Writing database file contents")
    }

    pub fn game_exists(&self, game_info: &GameInfo) -> bool {
        let game_id = game_info.get_id();
        self.games.iter().any(|game| game.id == game_id)
    }
}
