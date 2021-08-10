use serde::{Deserialize, Serialize};

use crate::blunder::Blunder;

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: String,
    pub blunders: Vec<Blunder>,
}
