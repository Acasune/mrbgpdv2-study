use crate::packets::{keepalive::KeepaliveMessage, open::OpenMessage, update::UpdateMessage};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Event {
    ManualStart,
    TcpConnectionConfirmed,
    BgpOpen(OpenMessage),
    KeepAliveMsg(KeepaliveMessage),
    UpdateMsg(UpdateMessage),
    Established,
    LocRib,
    LocRibChanged,
    AdjRibOutChanged,
    AdjRibInChanged,
}
