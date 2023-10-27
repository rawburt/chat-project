use std::fmt::Display;

pub trait Message: Display {}

#[derive(Debug, PartialEq)]
pub enum OutgoingMsg {
    // SAID(<from>, <message>)
    SaidUser(String, String),
    // SAID(<room>, <from>, <message>)
    SaidRoom(String, String, String),
    // ROOM(<room-name>)
    Room(String),
    // USER(<user-name>)
    User(String),
}

impl Message for OutgoingMsg {}

impl std::fmt::Display for OutgoingMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SaidUser(from, message) => write!(f, "{} SAID {}", from, message),
            Self::SaidRoom(room, from, message) => write!(f, "{} {} SAID {}", room, from, message),
            Self::Room(room) => write!(f, "ROOM {}", room),
            Self::User(name) => write!(f, "USER {}", name),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum IncomingMsg {
    /// NAME <user-name>
    Name(String),
    /// JOIN <room-name>
    Join(String),
    /// LEAVE <room-name>
    Leave(String),
    /// SAY <room-name> <message>
    SayRoom(String, String),
    /// SAY <user-name> <message>
    SayUser(String, String),
    /// USERS <room-name>
    Users(String),
    /// ROOMS
    Rooms,
    /// QUIT
    Quit,
}
