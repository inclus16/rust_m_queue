use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RawCommand
{
    sender: u8,
    data: Vec<u8>,
}

impl RawCommand {
    pub fn sender(&self) -> u8 {
        self.sender
    }

    pub fn data(self) -> Vec<u8> {
        self.data
    }
}