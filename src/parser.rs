use std::fmt::Display;

use crate::messages::{IncomingMsg, Message};
use regex::Regex;

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"^@[A-Za-z0-9\-\_]{3,20}$").unwrap();
    static ref ROOM_REGEX: Regex = Regex::new(r"^#[A-Za-z0-9\-\_]{3,20}$").unwrap();
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Name,
    Join,
    Leave,
    Say,
    Users,
    Rooms,
    Pong,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name => write!(f, "Name"),
            Self::Join => write!(f, "Join"),
            Self::Leave => write!(f, "Leave"),
            Self::Say => write!(f, "Say"),
            Self::Users => write!(f, "Users"),
            Self::Rooms => write!(f, "Rooms"),
            Self::Pong => write!(f, "Pong"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The given name doesn't match the required format.
    BadNameFormat,
    /// The given room name doesn't match the required format.
    BadRoomNameFormat,
    /// The incoming message doesn't have a correct amount of arguments.
    BadArguments,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadArguments => write!(f, "ERROR bad arguments"),
            Self::BadNameFormat => write!(f, "ERROR bad name format"),
            Self::BadRoomNameFormat => write!(f, "ERROR bad room name format"),
        }
    }
}

impl Message for ParseError {}

#[derive(Debug, PartialEq)]
pub enum ParsedAction {
    /// Ignore the input.
    None,
    /// Process a well-formed message.
    Process(IncomingMsg),
    /// Error parsing a valid command.
    Error(Command, ParseError),
}

impl Display for ParsedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "ParsedAction::None"),
            Self::Process(msg) => write!(f, "ParsedAction::Process({})", msg),
            Self::Error(cmd, err) => write!(f, "ParsedAction::Error({}, {})", cmd, err),
        }
    }
}

pub fn parse_incoming(input: &str) -> ParsedAction {
    if input.is_empty() {
        return ParsedAction::None;
    }

    let pieces: Vec<&str> = input.split(' ').collect();

    match pieces[0] {
        "QUIT" => ParsedAction::Process(IncomingMsg::Quit),
        "NAME" => {
            if pieces.len() == 2 {
                if NAME_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Name(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(Command::Name, ParseError::BadNameFormat)
                }
            } else {
                ParsedAction::Error(Command::Name, ParseError::BadArguments)
            }
        }
        "JOIN" => {
            if pieces.len() == 2 {
                if ROOM_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Join(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(Command::Join, ParseError::BadRoomNameFormat)
                }
            } else {
                ParsedAction::Error(Command::Join, ParseError::BadArguments)
            }
        }
        "LEAVE" => {
            if pieces.len() == 2 {
                if ROOM_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Leave(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(Command::Leave, ParseError::BadRoomNameFormat)
                }
            } else {
                ParsedAction::Error(Command::Leave, ParseError::BadArguments)
            }
        }
        "SAY" => {
            if pieces.len() >= 3 {
                if ROOM_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::SayRoom(
                        pieces[1].to_string(),
                        pieces[2..].join(" "),
                    ))
                } else if NAME_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::SayUser(
                        pieces[1].to_string(),
                        pieces[2..].join(" "),
                    ))
                } else {
                    let next = pieces[1].chars().next().unwrap();
                    if next == '#' {
                        ParsedAction::Error(Command::Say, ParseError::BadRoomNameFormat)
                    } else if next == '@' {
                        ParsedAction::Error(Command::Say, ParseError::BadNameFormat)
                    } else {
                        ParsedAction::Error(Command::Say, ParseError::BadArguments)
                    }
                }
            } else {
                ParsedAction::Error(Command::Say, ParseError::BadArguments)
            }
        }
        "ROOMS" => {
            if pieces.len() == 1 {
                ParsedAction::Process(IncomingMsg::Rooms)
            } else {
                ParsedAction::Error(Command::Rooms, ParseError::BadArguments)
            }
        }
        "USERS" => {
            if pieces.len() == 2 {
                if ROOM_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Users(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(Command::Users, ParseError::BadRoomNameFormat)
                }
            } else {
                ParsedAction::Error(Command::Users, ParseError::BadArguments)
            }
        }
        "PONG" => {
            if pieces.len() == 1 {
                ParsedAction::Process(IncomingMsg::Pong)
            } else {
                ParsedAction::Error(Command::Pong, ParseError::BadArguments)
            }
        }
        // ignore unknown commands
        _ => ParsedAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_regex() {
        // good
        assert!(NAME_REGEX.is_match("@robert"));
        assert!(NAME_REGEX.is_match("@rgp"));
        // bad
        assert!(!NAME_REGEX.is_match("@012345678901234567891"));
        assert!(!NAME_REGEX.is_match("@gj"));
    }

    #[test]
    fn test_parse_incoming_empty_input() {
        assert_eq!(parse_incoming(""), ParsedAction::None);
    }

    #[test]
    fn test_parse_incoming_quit() {
        assert_eq!(
            parse_incoming("QUIT"),
            ParsedAction::Process(IncomingMsg::Quit)
        );
        assert_eq!(
            parse_incoming("QUIT other stuff"),
            ParsedAction::Process(IncomingMsg::Quit)
        );
        assert_eq!(parse_incoming("quit other stuff"), ParsedAction::None);
        assert_eq!(parse_incoming("quit"), ParsedAction::None);
    }

    #[test]
    fn test_parse_incoming_name() {
        assert_eq!(
            parse_incoming("NAME @robert"),
            ParsedAction::Process(IncomingMsg::Name("@robert".to_string()))
        );
        assert_eq!(
            parse_incoming("NAME"),
            ParsedAction::Error(Command::Name, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("NAME @robert Steve"),
            ParsedAction::Error(Command::Name, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("NAME @robert**"),
            ParsedAction::Error(Command::Name, ParseError::BadNameFormat)
        );
        assert_eq!(parse_incoming("name"), ParsedAction::None);
    }

    #[test]
    fn test_parse_incoming_join() {
        assert_eq!(
            parse_incoming("JOIN #room1"),
            ParsedAction::Process(IncomingMsg::Join("#room1".to_string()))
        );
        assert_eq!(
            parse_incoming("JOIN"),
            ParsedAction::Error(Command::Join, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("JOIN #room #room2"),
            ParsedAction::Error(Command::Join, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("JOIN @room"),
            ParsedAction::Error(Command::Join, ParseError::BadRoomNameFormat)
        );
        assert_eq!(parse_incoming("join"), ParsedAction::None);
    }

    #[test]
    fn test_parse_incoming_leave() {
        assert_eq!(
            parse_incoming("LEAVE #room1"),
            ParsedAction::Process(IncomingMsg::Leave("#room1".to_string()))
        );
        assert_eq!(
            parse_incoming("LEAVE"),
            ParsedAction::Error(Command::Leave, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("LEAVE #room #room2"),
            ParsedAction::Error(Command::Leave, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("LEAVE @room"),
            ParsedAction::Error(Command::Leave, ParseError::BadRoomNameFormat)
        );
        assert_eq!(parse_incoming("leave"), ParsedAction::None);
    }

    #[test]
    fn test_parse_incoming_say() {
        assert_eq!(
            parse_incoming("SAY #room341 hello everyone!"),
            ParsedAction::Process(IncomingMsg::SayRoom(
                "#room341".to_string(),
                "hello everyone!".to_string()
            ))
        );
        assert_eq!(
            parse_incoming("SAY @kelsey hi kelsey :)"),
            ParsedAction::Process(IncomingMsg::SayUser(
                "@kelsey".to_string(),
                "hi kelsey :)".to_string()
            ))
        );
        assert_eq!(
            parse_incoming("SAY #room++ hi there room!"),
            ParsedAction::Error(Command::Say, ParseError::BadRoomNameFormat)
        );
        assert_eq!(
            parse_incoming("SAY @friend% hi there friend!"),
            ParsedAction::Error(Command::Say, ParseError::BadNameFormat)
        );
        assert_eq!(
            parse_incoming("SAY @dave"),
            ParsedAction::Error(Command::Say, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("SAY #happy"),
            ParsedAction::Error(Command::Say, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("SAY "),
            ParsedAction::Error(Command::Say, ParseError::BadArguments)
        );
    }

    #[test]
    fn test_parse_incoming_rooms() {
        assert_eq!(
            parse_incoming("ROOMS"),
            ParsedAction::Process(IncomingMsg::Rooms)
        );
        assert_eq!(
            parse_incoming("ROOMS stuff"),
            ParsedAction::Error(Command::Rooms, ParseError::BadArguments)
        );
    }

    #[test]
    fn test_parse_incoming_users() {
        assert_eq!(
            parse_incoming("USERS #test123"),
            ParsedAction::Process(IncomingMsg::Users("#test123".to_string()))
        );
        assert_eq!(
            parse_incoming("USERS"),
            ParsedAction::Error(Command::Users, ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("USERS #juice #man"),
            ParsedAction::Error(Command::Users, ParseError::BadArguments)
        );
    }

    #[test]
    fn test_parse_incoming_pong() {
        assert_eq!(
            parse_incoming("PONG"),
            ParsedAction::Process(IncomingMsg::Pong)
        );
        assert_eq!(
            parse_incoming("PONG abc def"),
            ParsedAction::Error(Command::Pong, ParseError::BadArguments)
        );
    }
}
