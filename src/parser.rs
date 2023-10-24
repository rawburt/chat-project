use regex::Regex;

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"^@[A-Za-z0-9\-\_]{4,20}$").unwrap();
    static ref ROOM_REGEX: Regex = Regex::new(r"^#[A-Za-z0-9\-\_]{4,20}$").unwrap();
}

#[derive(Debug, PartialEq)]
enum IncomingMsg {
    /// NAME <user-name>
    Name(String),
    /// JOIN <room-name>
    Join(String),
    /// LEAVE
    Leave(String),
    /// SAY <room-name> <message>
    SayRoom(String, String),
    /// SAY <user-name> <message>
    SayUser(String, String),
    /// USERS <room-name>
    Users(String),
    /// ROOMS
    Rooms,
}

#[derive(Debug, PartialEq)]
enum ParseError {
    /// An error state that requires no message to the client.
    None,
    /// The given name doesn't match the required format.
    BadNameFormat,
    /// The given room name doesn't match the required format.
    BadRoomNameFormat,
    /// The incoming message doesn't have a correct amount of arguments.
    BadArguments,
}

#[derive(Debug, PartialEq)]
enum ParsedAction {
    /// Take no action
    None,
    /// Disconnect the client.
    Quit,
    /// Process a well-formed message.
    Process(IncomingMsg),
    /// Error parsing a valid command.
    Error(ParseError),
}

fn parse_incoming(input: &str) -> ParsedAction {
    if input.is_empty() {
        return ParsedAction::Error(ParseError::None);
    }

    let pieces: Vec<&str> = input.split(" ").collect();

    match pieces[0] {
        "QUIT" => ParsedAction::Quit,
        "NAME" => {
            if pieces.len() == 2 {
                if NAME_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Name(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(ParseError::BadNameFormat)
                }
            } else {
                ParsedAction::Error(ParseError::BadArguments)
            }
        }
        "JOIN" => {
            if pieces.len() == 2 {
                if ROOM_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Join(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(ParseError::BadRoomNameFormat)
                }
            } else {
                ParsedAction::Error(ParseError::BadArguments)
            }
        }
        "LEAVE" => {
            if pieces.len() == 2 {
                if ROOM_REGEX.is_match(pieces[1]) {
                    ParsedAction::Process(IncomingMsg::Leave(pieces[1].to_string()))
                } else {
                    ParsedAction::Error(ParseError::BadRoomNameFormat)
                }
            } else {
                ParsedAction::Error(ParseError::BadArguments)
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
                        ParsedAction::Error(ParseError::BadRoomNameFormat)
                    } else if next == '@' {
                        ParsedAction::Error(ParseError::BadNameFormat)
                    } else {
                        ParsedAction::Error(ParseError::BadArguments)
                    }
                }
            } else {
                ParsedAction::Error(ParseError::BadArguments)
            }
        }
        "ROOMS" => {
            if pieces.len() == 1 {
                ParsedAction::Process(IncomingMsg::Rooms)
            } else {
                ParsedAction::Error(ParseError::BadArguments)
            }
        }
        "USERS" => {
            if pieces.len() == 2 {
                if ROOM_REGEX.is_match(pieces[2]) {
                    ParsedAction::Process(IncomingMsg::Users(pieces[2].to_string()))
                } else {
                    ParsedAction::Error(ParseError::BadRoomNameFormat)
                }
            } else {
                ParsedAction::Error(ParseError::BadArguments)
            }
        }
        // ignore unknown commands
        _ => ParsedAction::Error(ParseError::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_incoming_empty_input() {
        assert_eq!(parse_incoming(""), ParsedAction::Error(ParseError::None));
    }

    #[test]
    fn test_parse_incoming_quit() {
        assert_eq!(parse_incoming("QUIT"), ParsedAction::Quit);
        assert_eq!(parse_incoming("QUIT other stuff"), ParsedAction::Quit);
        assert_eq!(
            parse_incoming("quit other stuff"),
            ParsedAction::Error(ParseError::None)
        );
        assert_eq!(
            parse_incoming("quit"),
            ParsedAction::Error(ParseError::None)
        );
    }

    #[test]
    fn test_parse_incoming_name() {
        assert_eq!(
            parse_incoming("NAME @robert"),
            ParsedAction::Process(IncomingMsg::Name("@robert".to_string()))
        );
        assert_eq!(
            parse_incoming("NAME"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("NAME @robert Steve"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("NAME @robert**"),
            ParsedAction::Error(ParseError::BadNameFormat)
        );
        assert_eq!(
            parse_incoming("name"),
            ParsedAction::Error(ParseError::None)
        );
    }

    #[test]
    fn test_parse_incoming_join() {
        assert_eq!(
            parse_incoming("JOIN #room1"),
            ParsedAction::Process(IncomingMsg::Join("#room1".to_string()))
        );
        assert_eq!(
            parse_incoming("JOIN"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("JOIN #room #room2"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("JOIN @room"),
            ParsedAction::Error(ParseError::BadRoomNameFormat)
        );
        assert_eq!(
            parse_incoming("join"),
            ParsedAction::Error(ParseError::None)
        );
    }

    #[test]
    fn test_parse_incoming_leave() {
        assert_eq!(
            parse_incoming("LEAVE #room1"),
            ParsedAction::Process(IncomingMsg::Leave("#room1".to_string()))
        );
        assert_eq!(
            parse_incoming("LEAVE"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("LEAVE #room #room2"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("LEAVE @room"),
            ParsedAction::Error(ParseError::BadRoomNameFormat)
        );
        assert_eq!(
            parse_incoming("leave"),
            ParsedAction::Error(ParseError::None)
        );
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
            ParsedAction::Error(ParseError::BadRoomNameFormat)
        );
        assert_eq!(
            parse_incoming("SAY @friend% hi there friend!"),
            ParsedAction::Error(ParseError::BadNameFormat)
        );
        assert_eq!(
            parse_incoming("SAY @dave"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("SAY #happy"),
            ParsedAction::Error(ParseError::BadArguments)
        );
        assert_eq!(
            parse_incoming("SAY "),
            ParsedAction::Error(ParseError::BadArguments)
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
            ParsedAction::Error(ParseError::BadArguments)
        );
    }
}
