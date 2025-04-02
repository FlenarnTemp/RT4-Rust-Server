use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::error::Error;
use std::time::Instant;
use crate::entity::block_walk::BlockWalk;
use crate::entity::entity::{Entity, EntityBehavior};
use crate::entity::entity_lifecycle::EntityLifeCycle;
use crate::entity::move_restrict::MoveRestrict;
use crate::entity::move_strategy::MoveStrategy;
use crate::entity::window_status::WindowStatus;
use crate::grid::coord_grid::CoordGrid;
use constants::window_mode::window_mode;
use log::debug;
use crate::engine::CLIENT_PROTOCOL_REPOSITORY;
use crate::entity::entity_type::EntityType;
use crate::entity::pathing_entity::PathingEntity;
use crate::entity::player_type::PlayerType;
use crate::game_connection::GameClient;
use crate::io::client::protocol::client_protocol::ClientProtocol;
use crate::io::client::protocol::client_protocol_category::{ClientProtocolCategory, ClientProtocolCategoryLimit};
use crate::io::client::protocol::client_protocol_repository::ClientProtocolRepository;
use crate::io::packet_pool::PacketPool;
use crate::io::server::model::if_opensub::If_OpenSub;
use crate::io::server::model::if_opentop::If_OpenTop;
use crate::io::server::model::rebuild_normal::RebuildNormal;
use crate::io::server::outgoing_message::{OutgoingMessage, OutgoingMessageEnum};
use crate::io::server::protocol::server_protocol_priority::ServerProtocolPriority;
use crate::io::server::protocol::server_protocol_repository::{ServerProtocolRepository, SERVER_PROTOCOL_REPOSITORY};
use crate::script::script_pointer::ScriptPointer;
use crate::script::script_provider::ScriptProvider;
use crate::script::script_runner::ScriptRunner;
use crate::script::script_state::ScriptState;
use crate::script::server_trigger_types::ServerTriggerTypes;

#[derive(Clone, PartialEq)]
pub struct Player {
    // Player type
    pub player_type: PlayerType,

    // Permanent
    pub pathing_entity: PathingEntity,
    pub move_restrict: MoveRestrict,
    pub block_walk: BlockWalk,
    pub move_strategy: MoveStrategy,
    pub gender: u8,
    pub playtime: i32,
    
    pid: usize,
    pub username: String,
    
    pub origin_coord: CoordGrid,
    
    // Client data
    pub client: GameClient,
    /// User packet limit
    pub user_limit: u8,
    /// Client packet limit
    pub client_limit: u8,
    pub restricted_limit: u8,

    pub user_path: Vec<i32>,
    pub op_called: bool,
    pub bytes_read: usize,


    pub window_status: WindowStatus,
    
    staff_mod_level: i32,
    
    pub request_logout: bool,
    pub request_idle_logout: bool,
    pub logging_out: bool,
    pub prevent_logout_until: i32,
    
    pub last_response: i32,
    pub last_connected: i32,
    pub verify_id: u16,
    
    pub protect: bool,  // Whether protected access is available.
    pub active_script: Option<Box<ScriptState>>,
    
    pub write_queue: VecDeque<Vec<u8>>,
}
impl Player {
    pub fn new(client: &mut Option<GameClient>, coord: CoordGrid, gender: u8, window_status: WindowStatus, staff_mod_level: i32, pid: usize, verify_id: u16, username: String) -> Player {
        Player {
            player_type: PlayerType::ClientBound,
            pathing_entity: PathingEntity::new(
                coord,
                1,
                1,
                EntityLifeCycle::FOREVER
            ),
            move_restrict: MoveRestrict::Normal,
            block_walk: BlockWalk::Npc,
            move_strategy: MoveStrategy::Smart,
            gender,
            playtime: -1,
            pid,
            username,
            origin_coord: CoordGrid { coord: 0 },
            staff_mod_level,
            client: GameClient::take_ownership(client),
            user_limit: 0,
            client_limit: 0,
            restricted_limit: 0,
            user_path: Vec::new(),
            op_called: false,
            bytes_read: 0,
            window_status,
            request_logout: false,
            request_idle_logout: false,
            logging_out: false,
            prevent_logout_until: -1,
            last_response: -1,
            last_connected: -1,
            verify_id,
            protect: false,
            active_script: None,
            write_queue: VecDeque::with_capacity(64),
        }
    }
    
    #[inline(always)]
    pub fn is_client_connected(&self) -> bool {
        if self.player_type == PlayerType::Headless {
            return false;
        }

        self.client.is_connection_active()
    }

    pub fn get_entity(&self) -> &Entity {
        &self.pathing_entity.entity
    }

    pub fn as_entity_type(self) -> EntityType {
        EntityType::Player(self)
    }
    
    pub(crate) fn get_coord(&self) -> CoordGrid {
        self.pathing_entity.coord()
    }

    pub(crate) fn set_coord(&mut self, coord: CoordGrid) {
        self.pathing_entity.set_coord(coord);
    }

    pub(crate) fn get_origin_coord(&self) -> CoordGrid {
        self.origin_coord
    }

    pub(crate) fn set_origin_coord(&mut self, coord: CoordGrid) {
        self.origin_coord = coord;
    }

    pub(crate) fn get_active(self) -> bool {
        self.pathing_entity.active()
    }
    
    pub(crate) fn set_active(&mut self, active: bool) {
        self.pathing_entity.set_active(active);
    }
    
    pub(crate) fn get_verify_id(&self) -> u16 {
        self.verify_id
    }
    
    pub(crate) fn get_incremented_verify_id(&mut self) -> u16 {
        self.verify_id = self.verify_id +1;
        self.verify_id
    }
    pub(crate) fn set_verify_id(&mut self, verify_id: u16) {
        self.verify_id = verify_id;
    }
    
    pub(crate) fn get_staff_mod_level(&self) -> i32 {
        self.staff_mod_level
    }
    
    pub(crate) fn get_pid(&self) -> usize {
        self.pid
    }
    
    pub(crate)fn set_pid(&mut self, pid: usize) {
        self.pid = pid;
    }
    
    pub fn delayed(&self) -> bool {
        self.pathing_entity.delayed
    }
    
    pub fn run_script(&mut self, mut script: ScriptState, protected: Option<bool>, force: Option<bool>) -> Result<i32, Box<dyn Error>>{
        let protect = protected.unwrap_or(false);
        let force = force.unwrap_or(false);
        
        if !force && protect && (self.protect || self.delayed()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Cannot get protected access for script: {}", script.script.name())
            )));
        }
        
        if protect {
            script.pointer_add(ScriptPointer::ProtectedActivePlayer);
            self.protect = true;
        }
        
        let state = ScriptRunner::execute(&mut script, false, false);
        
        if protect {
            self.protect = false;
        }
        
        if script.pointer_get(ScriptPointer::ProtectedActivePlayer) && script.active_player.is_some() {
            script.pointer_remove(ScriptPointer::ProtectedActivePlayer);
            if let Some(player) = script.active_player.as_mut() {
                player.protect = false;
            }
        }
        
        if script.pointer_get(ScriptPointer::ProtectedActivePlayer2) && script.active_player2.is_some() {
            script.pointer_remove(ScriptPointer::ProtectedActivePlayer2);
            if let Some(player) = script.active_player2.as_mut() {
                player.protect = false;
            }
        }

        Ok(state)
    }
    
    pub fn execute_script(&mut self, script: ScriptState, protected: Option<bool>, force: Option<bool>) {
        let protect = protected.unwrap_or(false);
        let force = force.unwrap_or(false);

        let mut script_clone = script.clone();

        let state = match self.run_script(script, Some(protect), Some(force)) {
            Ok(state) => state,
            Err(err) => {
                debug!("Script execution failed: {}", err);
                return;
            }
        };

        if state != ScriptState::FINISHED && state != ScriptState::ABORTED {
            if state == ScriptState::WORLD_SUSPENDED {
                // TODO - Engine.enqueueScript
            } else if state == ScriptState::NPC_SUSPENDED {

            } else {
                if let Some(mut player) = script_clone.active_player.take() {
                    let boxed_script = Box::new(script_clone.clone());
                    player.active_script = Some(boxed_script);
                    player.protect = protect;
                    script_clone.active_player = Some(player);
                }
            }
        } else if self.active_script.as_ref().map_or(false, |active_script| &script_clone == active_script.as_ref()) {
            self.active_script = None;
            // TODO - close modal crap goes here
        }
    }

    #[inline(always)]
    fn read(&mut self) -> bool {
        if !self.client.has_available(1).unwrap() {
            return false
        }

        if self.client.opcode == 0 {
            self.client.read_packet_with_size(1).unwrap();

            if self.client.decryptor.is_some() {
                // TODO - ISAAC stuff
            } else {
                self.client.opcode = self.client.inbound.g1();
            }
        }

        if self.client.waiting == -1 {
            self.client.read_packet_with_size(1).unwrap();
            self.client.waiting = self.client.inbound.g1() as i32;
        } else if self.client.waiting == -2 {
            self.client.read_packet_with_size(2).unwrap();
            self.client.waiting = self.client.inbound.g2() as i32;

            // Avoid processing potentially malicious packets
            if self.client.waiting > 20000 {
                self.client.shutdown();
                return false;
            }
        }

        if !self.client.has_available(self.client.waiting as usize).unwrap() {
            return false;
        }

        self.client.read_packet_with_size(self.client.waiting as usize).unwrap();

        unsafe {
            let message = ClientProtocolRepository::get().get_message(ClientProtocol::try_from(self.client.opcode).unwrap(), self.client.inbound.data.clone());
            if message.is_some() {
                let handler = ClientProtocolRepository::get().get_handler(ClientProtocol::try_from(self.client.opcode).unwrap());
                
                let success: bool = handler.expect(format!("No handler found for opcode: {}", self.client.opcode).as_str()).handle(message, self);
            }
        }
        

        self.bytes_read += self.client.inbound.position;

        self.client.opcode = 0;
        true
    }

    /// initial_login
    ///
    /// rebuild_normal
    ///
    /// chat_filter_settings
    ///
    /// varp_reset
    ///
    /// varps
    ///
    /// inventories
    ///
    /// interfaces
    ///
    /// stats
    ///
    /// runweight
    ///
    /// runenergy
    ///
    /// reset animations
    ///
    /// social
    pub fn on_login(&mut self) {
        let start = Instant::now();

        // Process initial data
        self.initial_login_data();
        self.rebuild_normal(false);

        let window_id = if self.window_status.window_mode.is_resizeable() { 746 } else { 548 };

        // Get verification ID once and reuse
        let mut verify_id = self.get_incremented_verify_id();

        // Create and send top interface message
        self.write(If_OpenTop::new(window_id, false, verify_id));

        verify_id = self.get_incremented_verify_id();
        self.write(If_OpenSub::new(window_id, 100, 662, 1, verify_id));

        if let Some(trigger) = ScriptProvider::get_by_trigger_specific(ServerTriggerTypes::LOGIN, -1, -1) {
            let script = ScriptRunner::init(trigger, Some(self.clone().as_entity_type()), None, None);
            self.execute_script(script, Some(true), None);
        }

        self.set_active(true);

        if cfg!(debug_assertions) {
            debug!("Processed on login in: {:?}", start.elapsed());
        }
    }

    fn initial_login_data(&mut self) {
        self.client.outbound.p1(self.get_staff_mod_level()); // Staff mod level
        self.client.outbound.p1(0); // Blackmarks?
        self.client.outbound.p1(0); // Underage (false = 0)
        self.client.outbound.p1(0); // Parental Chat consent
        self.client.outbound.p1(0); // Parental Advert Consent
        self.client.outbound.p1(0); // Map Quick Chat
        self.client.outbound.p1(0); // Mouse Recorder
        self.client.outbound.p2(self.get_pid() as i32); // Player ID
        self.client.outbound.p1(1); // Player Member
        self.client.outbound.p1(1); // Members map
    }

    fn rebuild_normal(&mut self, reconnect: bool) {
        let origin_x = CoordGrid::zone(self.get_origin_coord().x()) as i16;
        let origin_z = CoordGrid::zone(self.get_origin_coord().z()) as i16;

        let reload_left_x = (origin_x - 4) << 3;
        let reload_right_x = (origin_x + 5) << 3;
        let reload_top_z = (origin_z + 5) << 3;
        let reload_bottom_z = (origin_z - 4) << 3;

        let current_x = self.coord().x();
        let current_z = self.coord().z();

        let needs_rebuild = reconnect ||
            current_x < reload_left_x as u16 ||
            current_z < reload_bottom_z as u16 ||
            current_x > (reload_right_x - 1) as u16 ||
            current_z > (reload_top_z - 1) as u16;

        if needs_rebuild {
            let rebuild_msg = RebuildNormal::new(
                CoordGrid::zone(current_x) as i32,
                CoordGrid::zone(current_z) as i32,
                self.get_coord().local_x(),
                self.get_coord().local_z()
            );

            self.write(rebuild_msg);

            self.set_origin_coord(self.get_coord());
        }
    }

    pub fn decode_in(&mut self, current_tick: i32) -> bool {
        // Reset state
        self.user_path.clear();
        self.op_called = false;

        if !self.is_client_connected() {
            return false;
        }

        self.last_connected = current_tick;

        self.user_limit = 0;
        self.client_limit = 0;
        self.restricted_limit = 0;

        let max_user = ClientProtocolCategoryLimit::USER_EVENT as u8;
        let max_client = ClientProtocolCategoryLimit::CLIENT_EVENT as u8;
        let max_restricted = ClientProtocolCategoryLimit::RESTRICTED_EVENT as u8;

        while self.user_limit < max_user &&
            self.client_limit < max_client &&
            self.restricted_limit < max_restricted {
            if !self.read() {
                break;
            }
        }

        if self.bytes_read > 0 {
            self.last_response = current_tick;
            self.bytes_read = 0;
        }

        true
    }

    #[inline(always)]
    pub fn write_inner(&mut self, message: OutgoingMessageEnum) {
        if !self.is_client_connected() {
            return;
        }
        message.write_self(self);
    }
    
    pub fn encode_out(&mut self) {
        if !self.is_client_connected() {
            return;
        }

        // TODO - modal refresh!

        let mut messages = std::mem::take(&mut self.outgoing_messages);

        // Process all messages
        for message in &messages {
            message.write_self(self);
        }

        messages.clear();
        self.outgoing_messages = messages;
    }

    #[inline(always)]
    pub fn write<T: OutgoingMessage + Into<OutgoingMessageEnum>>(&mut self, message: T) {
        if !self.is_client_connected() {
            return;
        }
        
        if message.priority() == ServerProtocolPriority::IMMEDIATE {
            message.write_self(self);
        } else {
            self.outgoing_messages.push(message.into());
        }
    }

    #[inline(always)]
    pub fn get_server_protocol_repository(&self) -> &'static ServerProtocolRepository {
        &SERVER_PROTOCOL_REPOSITORY
    }
}