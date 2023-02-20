use serde::{Deserialize,Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub enum Channel {
    Port,
    Starboard,
    Other
}
