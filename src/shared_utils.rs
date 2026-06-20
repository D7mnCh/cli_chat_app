use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, PartialEq)]
pub enum NameValidation {
    Empty,
    Reserved,
    Used,
    IllegalChar(char),

    Valid(String),
}

// This struct is not for message delivery, so no writing to streams or reading of course
#[derive(Clone, Debug)]
pub enum ServerMessage {
    Unknown(String),
    ValidName(String),
    History(Vec<String>),
    ClientConnected(String),
    ClientDisconnected(String),
    Chat { sender: String, content: String },

    InvalidName(NameValidation),
}

impl ServerMessage {
    pub fn deserialize(msg: String) -> Self {
        let msg: Vec<&str> = msg.splitn(4, ':').collect();

        match msg[..] {
            // handle invalid name
            ["server", "error", "reserved_name"] => Self::InvalidName(NameValidation::Reserved),
            ["server", "error", "used_name"] => Self::InvalidName(NameValidation::Used),
            ["server", "error", "empty_name"] => Self::InvalidName(NameValidation::Empty),
            ["server", "error", "illegalchar", character] => {
                if let Some(character) = character.chars().next() {
                    return Self::InvalidName(NameValidation::IllegalChar(character));
                } else {
                    // i'll return ":" as the deafult (for now)
                    return Self::InvalidName(NameValidation::IllegalChar(':'));
                }
            }

            ["server", "success", "valid_name", name] => Self::ValidName(name.to_string()),

            // handle events
            ["server", "event", "client_connected", client_name] => {
                Self::ClientConnected(client_name.to_string())
            }
            ["server", "event", "client_disconnected", client_name] => {
                Self::ClientDisconnected(client_name.to_string())
            }

            ["client", "chat", sender, content] if !content.is_empty() => Self::Chat {
                sender: sender.to_string(),
                content: content.to_string(),
            },

            ref unknown => {
                let sliced_msg = unknown.iter();
                let mut string_msg = String::new();
                for str_msg in sliced_msg {
                    string_msg.push_str(str_msg);
                }
                Self::Unknown(string_msg)
            }
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            Self::InvalidName(namevalidation) => match namevalidation {
                // handling invalid name
                NameValidation::Used => {
                    format!("server:error:used_name\n")
                }
                NameValidation::Reserved => {
                    format!("server:error:reserved_name\n")
                }
                NameValidation::Empty => {
                    format!("server:error:empty_name\n")
                }
                NameValidation::IllegalChar(c) => {
                    format!("server:error:illegalchar:{c}\n")
                }
                _ => unreachable!(
                    "caller should not call ValidName when invoking InvalidName variant of ServerMessage enum "
                ),
            },

            Self::ValidName(client_name) => {
                format!("server:success:valid_name:{client_name}\n")
            }

            // events
            Self::ClientConnected(client_name) => {
                format!("server:event:client_connected:{client_name}\n")
            }
            Self::ClientDisconnected(client_name) => {
                format!("server:event:client_disconnected:{client_name}\n")
            }

            Self::Chat { sender, content } => format!("client:chat:{sender}:{content}\n"),

            Self::History(messages) => {
                let mut serialize_msg = String::new();
                for msg in messages.to_owned().iter_mut() {
                    serialize_msg.push_str(&(msg.to_owned() + "\n"));
                }
                serialize_msg
            }

            Self::Unknown(_msg) => {
                unreachable!("the caller must not invoke those variant on the server side")
            }
        }
    }
}

pub trait LockClean<'a, T> {
    fn lock_mutex(&'a self) -> MutexGuard<'a, T>;
}

impl<'a, T> LockClean<'a, T> for Mutex<T> {
    fn lock_mutex(&'a self) -> MutexGuard<'a, T> {
        // i can get poisoning data (not complete)
        //for now, just returns the data even though she might be currepted
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}
