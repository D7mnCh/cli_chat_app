use std::sync::mpsc::{self, Receiver, Sender};

use crate::{client::network::ServerState, shared_utils::NameValidation};

pub struct ChannelReceivers {
    pub server_state_rx: Receiver<ServerState>,
    pub name_validation_rx: Receiver<NameValidation>,
    pub new_message_rx: Receiver<bool>,
}

#[derive(Clone)]
pub struct ChannelSenders {
    pub server_state_tx: Sender<ServerState>,
    pub name_validation_tx: Sender<NameValidation>,
    pub new_message_tx: Sender<bool>,
}

pub fn create_channels() -> (ChannelSenders, ChannelReceivers) {
    let (server_state_tx, server_state_rx) = mpsc::channel::<ServerState>();
    let (name_validation_tx, name_validation_rx) = mpsc::channel::<NameValidation>();
    let (new_message_tx, new_message_rx) = mpsc::channel::<bool>();
    (
        ChannelSenders {
            server_state_tx,
            name_validation_tx,
            new_message_tx,
        },
        ChannelReceivers {
            server_state_rx,
            name_validation_rx,
            new_message_rx,
        },
    )
}
