use crate::messages::{Message, OutgoingMsg};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct User {
    sender: UnboundedSender<OutgoingMsg>,
    rooms: HashSet<String>,
}

impl User {
    pub fn new(sender: UnboundedSender<OutgoingMsg>) -> Self {
        Self {
            sender,
            rooms: HashSet::new(),
        }
    }

    pub fn add_room(&mut self, name: String) {
        self.rooms.insert(name);
    }

    pub fn remove_room(&mut self, name: &str) {
        self.rooms.remove(name);
    }

    pub fn send(&self, message: OutgoingMsg) -> Result<(), String> {
        match self.sender.send(message) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Room {
    users: HashSet<String>,
}

impl Room {
    pub fn new() -> Self {
        Self {
            users: HashSet::new(),
        }
    }

    pub fn add_user(&mut self, name: String) {
        self.users.insert(name);
    }

    pub fn remove_user(&mut self, name: &str) -> bool {
        self.users.remove(name)
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerError {
    RoomUnknown(String),
    UserAlreadyExists(String),
    // UserNotInRoom(<user-name>, <room-name>)
    UserNotInRoom(String, String),
    UserUnknown(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoomUnknown(name) => write!(f, "ERROR room unknown {}", name),
            Self::UserAlreadyExists(name) => write!(f, "ERROR user already exists {}", name),
            Self::UserNotInRoom(user_name, room_name) => {
                write!(f, "ERROR user not in room {} {}", user_name, room_name)
            }
            Self::UserUnknown(name) => write!(f, "ERROR user unknown {}", name),
        }
    }
}

impl Message for ServerError {}

impl std::error::Error for ServerError {}

#[derive(Debug)]
pub struct ServerState {
    users: HashMap<String, User>,
    rooms: HashMap<String, Room>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, name: String, user: User) -> Result<(), ServerError> {
        if self.users.contains_key(&name) {
            return Err(ServerError::UserAlreadyExists(name));
        }
        self.users.insert(name, user);
        Ok(())
    }

    pub fn remove_user(&mut self, name: &str) -> Result<(), ServerError> {
        match self.users.remove(name) {
            Some(user) => {
                // remove user from rooms
                for room_name in user.rooms {
                    self.leave_room(&room_name, name)?;
                }
                Ok(())
            }
            None => Err(ServerError::UserUnknown(name.to_string())),
        }
    }

    pub fn join_room(&mut self, room_name: String, user_name: String) -> Result<(), ServerError> {
        if !self.users.contains_key(&user_name) {
            return Err(ServerError::UserUnknown(user_name));
        }
        if let Some(room) = self.rooms.get_mut(&room_name) {
            // add user to existing room
            room.add_user(user_name.clone());
        } else {
            // create new room
            let mut room = Room::new();
            room.add_user(user_name.clone());
            self.rooms.insert(room_name.clone(), room);
        }
        // add room to user record
        if let Some(user) = self.users.get_mut(&user_name) {
            user.add_room(room_name);
        }
        Ok(())
    }

    pub fn leave_room(&mut self, room_name: &str, user_name: &str) -> Result<(), ServerError> {
        if let Some(room) = self.rooms.get_mut(room_name) {
            if room.remove_user(user_name) {
                // delete rooms that are empty
                if room.is_empty() {
                    self.rooms.remove(room_name);
                }
                // remove room from user record
                if let Some(user) = self.users.get_mut(user_name) {
                    user.remove_room(room_name);
                }
                Ok(())
            } else {
                Err(ServerError::UserNotInRoom(
                    user_name.to_string(),
                    room_name.to_string(),
                ))
            }
        } else {
            Err(ServerError::RoomUnknown(room_name.to_string()))
        }
    }

    pub fn rename_user(&mut self, old_name: &str, new_name: &str) -> Result<(), ServerError> {
        if let Some(user) = self.users.remove(old_name) {
            // rename user in each room the user is in
            for room_name in &user.rooms {
                if let Some(room) = self.rooms.get_mut(room_name) {
                    room.remove_user(old_name);
                    room.add_user(new_name.to_string());
                }
            }
            // rename user in main user list
            self.users.insert(new_name.to_string(), user);
            Ok(())
        } else {
            Err(ServerError::UserUnknown(old_name.to_string()))
        }
    }

    pub fn rooms(&self) -> Vec<String> {
        self.rooms.keys().map(|k| k.to_string()).collect()
    }

    pub fn users(&self, room_name: &str) -> Result<Vec<String>, ServerError> {
        if let Some(room) = self.rooms.get(room_name) {
            Ok(room.users.iter().map(|u| u.to_string()).collect())
        } else {
            Err(ServerError::RoomUnknown(room_name.to_string()))
        }
    }

    pub fn say_to_user(
        &self,
        from_user: &str,
        to_user: &str,
        message: String,
    ) -> Result<(), ServerError> {
        if let Some(to) = self.users.get(to_user) {
            // TODO: better errors
            to.send(OutgoingMsg::SaidUser(from_user.to_string(), message))
                .unwrap();
            Ok(())
        } else {
            Err(ServerError::UserUnknown(to_user.to_string()))
        }
    }

    pub fn say_to_room(
        &mut self,
        user_name: &str,
        room_name: &str,
        message: String,
    ) -> Result<(), ServerError> {
        if let Some(room) = self.rooms.get(room_name) {
            for room_user_name in &room.users {
                if room_user_name != user_name {
                    if let Some(user) = self.users.get_mut(room_user_name) {
                        // TODO: better errors
                        user.send(OutgoingMsg::SaidRoom(
                            room_name.to_string(),
                            user_name.to_string(),
                            message.clone(),
                        ))
                        .unwrap();
                    }
                }
            }
            Ok(())
        } else {
            Err(ServerError::RoomUnknown(room_name.to_string()))
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::{self, error::TryRecvError};

    #[test]
    fn test_server_state_add_user() {
        let mut state = ServerState::new();
        assert!(state.users.get("@robert").is_none());
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender))
            .is_ok());
        assert!(state.users.get("@robert").is_some());
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert_eq!(
            state.add_user("@robert".to_string(), User::new(sender)),
            Err(ServerError::UserAlreadyExists("@robert".to_string()))
        );
    }

    #[test]
    fn test_server_state_remove_user() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender))
            .is_ok());
        assert!(state.users.get("@robert").is_some());
        assert!(state.remove_user("@robert").is_ok());
        assert!(state.users.get("@robert").is_none());
        assert_eq!(
            state.remove_user("@robert"),
            Err(ServerError::UserUnknown("@robert".to_string()))
        );
    }

    #[test]
    fn test_server_state_join_room() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender))
            .is_ok());

        // join room that does not exist with user that exists
        assert!(!state.rooms.contains_key("#testroom"));
        assert!(state
            .join_room("#testroom".to_string(), "@robert".to_string())
            .is_ok());
        assert!(state.rooms.contains_key("#testroom"));
        assert!(state
            .users
            .get("@robert")
            .unwrap()
            .rooms
            .contains("#testroom"));

        // join room that exists with user that exists
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#testroom".to_string(), "@kelsey".to_string())
            .is_ok());
        assert!(state
            .users
            .get("@kelsey")
            .unwrap()
            .rooms
            .contains("#testroom"));

        // join room that does not exist with user that does not exist
        assert_eq!(
            state.join_room("#none".to_string(), "@notreal".to_string()),
            Err(ServerError::UserUnknown("@notreal".to_string()))
        );

        // join room that exists with user that does not exist
        assert_eq!(
            state.join_room("#testroom".to_string(), "@fakey".to_string()),
            Err(ServerError::UserUnknown("@fakey".to_string()))
        );
    }

    #[test]
    fn test_server_state_leave_room() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#testroom".to_string(), "@robert".to_string())
            .is_ok());
        assert!(state.rooms.contains_key("#testroom"));
        assert!(state
            .users
            .get("@robert")
            .unwrap()
            .rooms
            .contains("#testroom"));
        assert!(state.leave_room("#testroom", "@robert").is_ok());
        assert!(!state
            .users
            .get("@robert")
            .unwrap()
            .rooms
            .contains("#testroom"));

        // last user to leave room makes room go away
        assert!(!state.rooms.contains_key("#testroom"));

        // room does not exist
        assert_eq!(
            state.leave_room("#fakeroom", "@robert"),
            Err(ServerError::RoomUnknown("#fakeroom".to_string()))
        );

        // user does not exist
        assert!(state
            .join_room("#testroom".to_string(), "@robert".to_string())
            .is_ok());

        assert_eq!(
            state.leave_room("#testroom", "@kelsey"),
            Err(ServerError::UserNotInRoom(
                "@kelsey".to_string(),
                "#testroom".to_string()
            ))
        );
    }

    #[test]
    fn test_server_state_remove_user_leaves_room() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#applejuice".to_string(), "@kelsey".to_string())
            .is_ok());
        assert!(state.rooms.contains_key("#applejuice"));
        assert!(state.users.contains_key("@kelsey"));
        assert!(state.remove_user("@kelsey").is_ok());
        assert!(!state.rooms.contains_key("#applejuice"));
        assert!(!state.users.contains_key("@kelsey"));
    }

    #[test]
    fn test_server_state_rename_user() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#applejuice".to_string(), "@kelsey".to_string())
            .is_ok());
        assert!(state
            .join_room("#testing123".to_string(), "@kelsey".to_string())
            .is_ok());

        // initial state
        assert!(state.rooms.contains_key("#applejuice"));
        assert!(state.rooms.contains_key("#testing123"));

        assert!(state
            .users
            .get("@kelsey")
            .unwrap()
            .rooms
            .contains("#applejuice"));
        assert!(state
            .users
            .get("@kelsey")
            .unwrap()
            .rooms
            .contains("#testing123"));

        assert!(state
            .rooms
            .get("#applejuice")
            .unwrap()
            .users
            .contains("@kelsey"));
        assert!(state
            .rooms
            .get("#testing123")
            .unwrap()
            .users
            .contains("@kelsey"));

        // renamed state
        assert!(state.rename_user("@kelsey", "@littleb1t").is_ok());

        assert!(state.users.get("@kelsey").is_none());
        assert!(state
            .users
            .get("@littleb1t")
            .unwrap()
            .rooms
            .contains("#applejuice"));
        assert!(state
            .users
            .get("@littleb1t")
            .unwrap()
            .rooms
            .contains("#testing123"));

        assert!(state
            .rooms
            .get("#applejuice")
            .unwrap()
            .users
            .contains("@littleb1t"));
        assert!(state
            .rooms
            .get("#testing123")
            .unwrap()
            .users
            .contains("@littleb1t"));
    }

    #[test]
    fn test_server_state_rename_user_bad() {
        let mut state = ServerState::new();
        assert_eq!(
            state.rename_user("@robert", "@bobert"),
            Err(ServerError::UserUnknown("@robert".to_string()))
        );
    }

    #[test]
    fn test_server_state_rooms() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#applejuice".to_string(), "@kelsey".to_string())
            .is_ok());
        assert!(state
            .join_room("#testing123".to_string(), "@kelsey".to_string())
            .is_ok());
        assert!(state
            .join_room("#room_123".to_string(), "@kelsey".to_string())
            .is_ok());

        let mut rooms = state.rooms();
        rooms.sort();
        let mut expected = vec![
            "#applejuice".to_string(),
            "#testing123".to_string(),
            "#room_123".to_string(),
        ];
        expected.sort();
        assert_eq!(rooms, expected);
    }

    #[test]
    fn test_server_state_users() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#applejuice".to_string(), "@kelsey".to_string())
            .is_ok());
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#applejuice".to_string(), "@robert".to_string())
            .is_ok());
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@ch4ch4".to_string(), User::new(sender))
            .is_ok());
        assert!(state
            .join_room("#applejuice".to_string(), "@ch4ch4".to_string())
            .is_ok());

        let mut users = state.users("#applejuice").unwrap();
        users.sort();
        let mut expected = vec![
            "@ch4ch4".to_string(),
            "@robert".to_string(),
            "@kelsey".to_string(),
        ];
        expected.sort();
        assert_eq!(users, expected);

        assert_eq!(
            state.users("#notrealroom"),
            Err(ServerError::RoomUnknown("#notrealroom".to_string()))
        );
    }

    #[tokio::test]
    async fn test_server_state_say_to_user() {
        let mut state = ServerState::new();
        let (sender_kelsey, mut receiver_kelsey) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender_kelsey))
            .is_ok());
        let (sender_robert, _receiver_robert) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender_robert))
            .is_ok());

        assert_eq!(Err(TryRecvError::Empty), receiver_kelsey.try_recv());

        assert!(state
            .say_to_user("@robert", "@kelsey", "hi there! how are you?".to_string())
            .is_ok());

        assert_eq!(
            Some(OutgoingMsg::SaidUser(
                "@robert".to_string(),
                "hi there! how are you?".to_string()
            )),
            receiver_kelsey.recv().await
        );

        assert_eq!(
            state.say_to_user("@robert", "@notreal", "uhoh!!!!??!!".to_string()),
            Err(ServerError::UserUnknown("@notreal".to_string()))
        );
    }

    #[tokio::test]
    async fn test_server_state_say_to_room() {
        let mut state = ServerState::new();
        let (sender_kelsey, mut receiver_kelsey) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User::new(sender_kelsey))
            .is_ok());
        let (sender_robert, mut receiver_robert) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User::new(sender_robert))
            .is_ok());
        let (sender_dave, mut receiver_dave) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@dave".to_string(), User::new(sender_dave))
            .is_ok());

        assert!(state
            .join_room("#testroom".to_string(), "@kelsey".to_string())
            .is_ok());
        assert!(state
            .join_room("#testroom".to_string(), "@robert".to_string())
            .is_ok());
        assert!(state
            .join_room("#testroom".to_string(), "@dave".to_string())
            .is_ok());

        assert!(state
            .say_to_room("@dave", "#testroom", "hello my room friends!".to_string())
            .is_ok());

        // dont send room message to self
        assert_eq!(Err(TryRecvError::Empty), receiver_dave.try_recv());
        assert_eq!(
            Some(OutgoingMsg::SaidRoom(
                "#testroom".to_string(),
                "@dave".to_string(),
                "hello my room friends!".to_string()
            )),
            receiver_kelsey.recv().await
        );
        assert_eq!(
            Some(OutgoingMsg::SaidRoom(
                "#testroom".to_string(),
                "@dave".to_string(),
                "hello my room friends!".to_string()
            )),
            receiver_robert.recv().await
        );

        assert_eq!(
            state.say_to_room("@dave", "#notreal", "hello my room friends!".to_string()),
            Err(ServerError::RoomUnknown("#notreal".to_string()))
        );
    }
}
