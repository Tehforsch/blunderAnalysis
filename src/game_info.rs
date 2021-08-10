use pgnparse::parser::PgnInfo;

pub struct GameInfo {
    pub info: PgnInfo,
}

impl GameInfo {
    pub fn get_id(&self) -> String {
        self.info.headers["Site"].clone()
    }
}
