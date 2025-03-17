use crate::io::packet::Packet;
use crate::io::server::codec::message_encoder::MessageEncoder;
use crate::io::server::model::if_opensub::If_OpenSub;
use crate::io::server::protocol::server_protocol::ServerProtocol;

pub struct If_OpenSubEncoder;

impl If_OpenSubEncoder {
    pub fn new() -> Self { If_OpenSubEncoder }
}

impl MessageEncoder<If_OpenSub> for If_OpenSubEncoder {
    fn protocol(&self) -> ServerProtocol { ServerProtocol::IF_OPENSUB }

    fn encode(&self, packet: &mut Packet, message: &If_OpenSub) {
        todo!()
    }
}