//! [OutgoingMsg] and [IncomingMsg] codify the messages that are sent between clients and the server. These messages
//! are defined by the chat protocol. ERROR messages are codified for each error type in various other modules.
//! 
use std::fmt::Display;

/// [Message] trait signifies to the rest of the code that a piece of data is able to be sent/received between
/// the clients and the server.
pub trait Message: Display {}

/// Messages that the server sends to clients.
#[derive(Debug, PartialEq, Clone)]
pub enum OutgoingMsg {
    /// PING
    Ping,
    /// CONNECTED
    Connected,
    /// REGISTERED
    Registered,
    /// SAID from message
    SaidUser(String, String),
    /// SAID room-name from message
    SaidRoom(String, String, String),
    /// ROOM room-name
    Room(String),
    /// USER user-name
    User(String),
    /// JOINED room-name user-name
    Joined(String, String),
    /// LEFT room-name user-name
    Left(String, String),
}

impl Message for OutgoingMsg {}

impl Display for OutgoingMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ping => write!(f, "PING"),
            Self::Connected => write!(f, "CONNECTED"),
            Self::Registered => write!(f, "REGISTERED"),
            Self::SaidUser(from, message) => write!(f, "{} SAID {}", from, message),
            Self::SaidRoom(room, from, message) => write!(f, "{} {} SAID {}", room, from, message),
            Self::Room(room) => write!(f, "ROOM {}", room),
            Self::User(name) => write!(f, "USER {}", name),
            Self::Joined(room, user) => write!(f, "{} {} JOINED", room, user),
            Self::Left(room, user) => write!(f, "{} {} LEFT", room, user),
        }
    }
}

/// Messages that the clients sends to the server.
#[derive(Debug, PartialEq)]
pub enum IncomingMsg {
    /// NAME user-name
    Name(String),
    /// JOIN room-name
    Join(String),
    /// LEAVE room-name
    Leave(String),
    /// SAY room-name message
    SayRoom(String, String),
    /// SAY user-name message
    SayUser(String, String),
    /// USERS room-name
    Users(String),
    /// ROOMS
    Rooms,
    /// QUIT
    Quit,
    /// PONG
    Pong,
}

impl Display for IncomingMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(name) => write!(f, "NAME {}", name),
            Self::Join(room) => write!(f, "JOIN {}", room),
            Self::Leave(room) => write!(f, "LEAVE {}", room),
            Self::SayRoom(room, message) => write!(f, "SAY {} {}", room, message),
            Self::SayUser(name, message) => write!(f, "SAY {} {}", name, message),
            Self::Users(room) => write!(f, "USERS {}", room),
            Self::Rooms => write!(f, "ROOMS"),
            Self::Quit => write!(f, "QUIT"),
            Self::Pong => write!(f, "PONG"),
        }
    }
}
