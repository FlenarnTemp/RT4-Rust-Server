use std::any::{Any, TypeId};
use std::collections::HashMap;
use crate::server::codec::message_encoder::MessageEncoder;
use crate::server::codec::rebuild_normal_encoder::RebuildNormalEncoder;
use crate::server::model::rebuild_normal::RebuildNormal;
use crate::server::outgoing_message::OutgoingMessage;

struct TypedEncoder<T: OutgoingMessage> {
    encoder: Box<dyn MessageEncoder<T> + Send + Sync>,
}

pub struct ServerProtocolRepository {
    encoders: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServerProtocolRepository {
    pub fn new() -> Self {
        let mut repository = ServerProtocolRepository {
            encoders: HashMap::new(),
        };
        
        repository.bind::<RebuildNormal>(RebuildNormalEncoder::new());
        
        repository
    }

    fn bind<T: 'static + OutgoingMessage>(&mut self, encoder: impl MessageEncoder<T> + 'static + Send + Sync) {
        let type_id = TypeId::of::<T>();
        
        if self.encoders.contains_key(&type_id) {
            panic!("[ServerProtocolRepository] Duplicate encoder");
        }
        
        let typed_encoder = TypedEncoder {
            encoder: Box::new(encoder),
        };
        self.encoders.insert(type_id, Box::new(typed_encoder));
    }

    pub fn get_encoder<T: 'static + OutgoingMessage>(&self, _: &T) -> Option<&(dyn MessageEncoder<T> + Send + Sync)> {
        let type_id = TypeId::of::<T>();
        self.encoders.get(&type_id)
            .and_then(|box_any| box_any.downcast_ref::<TypedEncoder<T>>())
            .map(|typed| typed.encoder.as_ref())
    }
}

lazy_static::lazy_static! {
    pub static ref SERVER_PROTOCOL_REPOSITORY: ServerProtocolRepository = ServerProtocolRepository::new();
}