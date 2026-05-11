// This module is not for message delivery, so no writing to streams or reading of course

use crate::shared_utils::NameValidation;

#[derive(Clone, Debug)]
pub enum ServerMessage {
    ValidName(String),
    History(Vec<String>),
    _NewConnection(String),
    ClientDisconnected(String),
    Chat { sender: String, content: String },

    InvalidName(NameValidation),
}

impl ServerMessage {
    pub fn serialize(&self) -> String {
        match self {
            Self::InvalidName(namevalidation) => match namevalidation {
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
                // can't reach cuz i didn't return ValidName() when i return InvalidName varaints
                _ => unreachable!(),
            },
            Self::ValidName(client_name) => {
                format!("server:success:valid_name:{client_name}\n")
            }
            Self::Chat { sender, content } => format!("client:chat:{sender}:{content}\n"),
            Self::_NewConnection(client_name) => format!("server:event:{client_name} connected\n"),
            Self::History(messages) => {
                let mut serialize_msg = String::new();
                for msg in messages.to_owned().iter_mut() {
                    serialize_msg.push_str(&(msg.to_owned() + "\n"));
                }
                serialize_msg
            }
            Self::ClientDisconnected(client_name) => {
                format!("server:event:{client_name} disconnected\n")
            }
        }
    }
}
