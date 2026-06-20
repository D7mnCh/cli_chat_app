pub enum ClientMessages {
    CheckName(String),
    Chat { sender: String, content: String },
}

impl ClientMessages {
    pub fn serialize(&self) -> String {
        match self {
            Self::CheckName(client_name) => format!("client:check_name:{client_name}\n"),
            Self::Chat { sender, content } => format!("client:chat:{sender}:{content}"),
        }
    }
}
