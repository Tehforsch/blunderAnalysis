use serde::{Deserialize, Serialize};

use crate::blunder::Blunder;

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub blunders: Vec<Blunder>,
}
