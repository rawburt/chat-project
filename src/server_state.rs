use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct User {
    sender: UnboundedSender<String>,
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
}

#[derive(Debug, PartialEq)]
pub enum ServerError {
    UserAlreadyExists(String),
    UserUnknown(String),
}

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
            Some(_) => Ok(()),
            None => Err(ServerError::UserUnknown(name.to_string())),
        }
    }

    pub fn join_room(&mut self, room_name: String, user_name: String) -> Result<(), ServerError> {
        if !self.users.contains_key(&user_name) {
            return Err(ServerError::UserUnknown(user_name));
        }
        if let Some(room) = self.rooms.get_mut(&room_name) {
            room.add_user(user_name);
        } else {
            let mut room = Room::new();
            room.add_user(user_name);
            self.rooms.insert(room_name, room);
        }
        Ok(())
    }

    // pub fn rename_user(&mut self, old_name: &str, new_name: String) -> Result<(), ServerError> {}

    // rename_user
    // join_room
    // leave_room
    // room_names
    // room_user_names
    // say_to_room
    // say_to_user
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_server_state_add_user() {
        let mut state = ServerState::new();
        assert!(state.users.get("@robert").is_none());
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User { sender })
            .is_ok());
        assert!(state.users.get("@robert").is_some());
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert_eq!(
            state.add_user("@robert".to_string(), User { sender }),
            Err(ServerError::UserAlreadyExists("@robert".to_string()))
        );
    }

    #[test]
    fn test_server_state_remove_user() {
        let mut state = ServerState::new();
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@robert".to_string(), User { sender })
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
            .add_user("@robert".to_string(), User { sender })
            .is_ok());

        // join room that does not exist with user that exists
        assert!(!state.rooms.contains_key("#testroom"));
        assert!(state
            .join_room("#testroom".to_string(), "@robert".to_string())
            .is_ok());
        assert!(state.rooms.contains_key("#testroom"));

        // join room that exists with user that exists
        let (sender, _receiver) = mpsc::unbounded_channel();
        assert!(state
            .add_user("@kelsey".to_string(), User { sender })
            .is_ok());
        assert!(state
            .join_room("#testroom".to_string(), "@kelsey".to_string())
            .is_ok());

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
}
