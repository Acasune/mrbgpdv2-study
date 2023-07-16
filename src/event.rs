use crate::packets::{keepalive::KeepaliveMessage, open::OpenMessage};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Event {
    ManualStart,
    TcpConnectionConfirmed,
    BgpOpen(OpenMessage),
    KeepAliveMsg(KeepaliveMessage),
}
