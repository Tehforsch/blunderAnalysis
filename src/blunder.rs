use serde::{Deserialize, Serialize};

use crate::evaluation::Evaluation;

#[derive(Serialize, Deserialize, Hash, Debug)]
pub struct Blunder {
    pub position: String,
    pub move_: String,
    pub eval_before: Evaluation,
    pub eval_after: Evaluation,
}

impl PartialEq for Blunder {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for Blunder {
    fn assert_receiver_is_total_eq(&self) {}
}
