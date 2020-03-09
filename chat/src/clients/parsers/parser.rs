

pub struct Parser {
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    ws_done_join_handle: tokio::task::JoinHandle<()>,
}

pub enum EventType {
    All,
    PrivateMsg,
}

impl Parser {
    pub fn listen(type: EventType) -> tokio::sync::mpsc::UnboundedResiver<Message> {

    }
}
