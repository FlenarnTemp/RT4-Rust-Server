use crate::entity::entity_lifecycle::EntityLifeCycle;
use crate::entity::network_player::NetworkPlayer;
use crate::entity::npc::NPC;
use crate::entity::player::Player;
use crate::grid::coord_grid::CoordGrid;

pub trait EntityBehavior {
    fn coord(&self) -> CoordGrid;
    fn width(&self) -> u8;
    fn length(&self) -> u8;
    fn lifecycle(&self) -> EntityLifeCycle;
    fn active(&self) -> bool;
    fn lifecycle_tick(&self) -> u32;
    fn last_lifecycle_tick(&self) -> u32;

    fn set_coord(&mut self, coord: CoordGrid);
    fn set_active(&mut self, active: bool);
    fn set_lifecycle_tick(&mut self, tick: u32);
    fn set_last_lifecycle_tick(&mut self, tick: u32);

    fn check_lifecycle(&self, tick: u32) -> bool {
        match self.lifecycle() {
            EntityLifeCycle::FOREVER => true,
            EntityLifeCycle::RESPAWN => self.lifecycle_tick() < tick,
            EntityLifeCycle::DESPAWN => self.lifecycle_tick() > tick,
        }
    }

    fn set_lifecycle(&mut self, tick: u32) {
        self.set_last_lifecycle_tick(self.lifecycle_tick());
        self.set_lifecycle_tick(tick);
    }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct Entity {
    pub coord: CoordGrid,
    pub width: u8,
    pub length: u8,
    pub lifecycle: EntityLifeCycle,
    pub active: bool,

    pub lifecycle_tick: u32,
    pub last_lifecycle_tick: u32,
}

impl Entity {
    pub fn new(coord: CoordGrid, width: u8, length: u8, lifecycle: EntityLifeCycle) -> Self {
        Entity {
            coord,
            width,
            length,
            lifecycle,
            lifecycle_tick: 0,
            last_lifecycle_tick: 0,
            active: false,
        }
    }
}

impl EntityBehavior for Entity {
    fn coord(&self) -> CoordGrid { self.coord }
    fn width(&self) -> u8 { self.width }
    fn length(&self) -> u8 { self.length }
    fn lifecycle(&self) -> EntityLifeCycle { self.lifecycle }
    fn active(&self) -> bool { self.active }
    fn lifecycle_tick(&self) -> u32 { self.lifecycle_tick }
    fn last_lifecycle_tick(&self) -> u32 { self.last_lifecycle_tick }

    fn set_coord(&mut self, coord: CoordGrid) { self.coord = coord; }
    fn set_active(&mut self, active: bool) { self.active = active; }
    fn set_lifecycle_tick(&mut self, tick: u32) { self.lifecycle_tick = tick; }
    fn set_last_lifecycle_tick(&mut self, tick: u32) { self.last_lifecycle_tick = tick; }
}

macro_rules! impl_entity_behavior_for {
    ($type:ty, field: $field:ident) => {
        impl EntityBehavior for $type {
            fn coord(&self) -> CoordGrid { self.$field.coord }
            fn width(&self) -> u8 { self.$field.width }
            fn length(&self) -> u8 { self.$field.length }
            fn lifecycle(&self) -> EntityLifeCycle { self.$field.lifecycle }
            fn active(&self) -> bool { self.$field.active }
            fn lifecycle_tick(&self) -> u32 { self.$field.lifecycle_tick }
            fn last_lifecycle_tick(&self) -> u32 { self.$field.last_lifecycle_tick }
            
            fn set_coord(&mut self, coord: CoordGrid) { self.$field.coord = coord; }
            fn set_active(&mut self, active: bool) { self.$field.active = active; }
            fn set_lifecycle_tick(&mut self, tick: u32) { self.$field.lifecycle_tick = tick; }
            fn set_last_lifecycle_tick(&mut self, tick: u32) { self.$field.last_lifecycle_tick = tick; }
        }
    };
    
    ($type:ty, delegate: $($field:ident).+) => {
        impl EntityBehavior for $type {
            fn coord(&self) -> CoordGrid { 
                self.$($field).+.coord()
            }
            
            fn width(&self) -> u8 { 
                self.$($field).+.width()
            }
            
            fn length(&self) -> u8 { 
                self.$($field).+.length()
            }
            
            fn lifecycle(&self) -> EntityLifeCycle { 
                self.$($field).+.lifecycle()
            }
            
            fn active(&self) -> bool { 
                self.$($field).+.active()
            }
            
            fn lifecycle_tick(&self) -> u32 { 
                self.$($field).+.lifecycle_tick()
            }
            
            fn last_lifecycle_tick(&self) -> u32 { 
                self.$($field).+.last_lifecycle_tick()
            }
            
            fn set_coord(&mut self, coord: CoordGrid) { 
                self.$($field).+.set_coord(coord);
            }
            
            fn set_active(&mut self, active: bool) { 
                self.$($field).+.set_active(active);
            }
            
            fn set_lifecycle_tick(&mut self, tick: u32) { 
                self.$($field).+.set_lifecycle_tick(tick);
            }
            
            fn set_last_lifecycle_tick(&mut self, tick: u32) { 
                self.$($field).+.set_last_lifecycle_tick(tick);
            }
        }
    };
}

#[derive(Copy, Clone)]
pub struct PathingEntity {
    entity: Entity,
}

impl PathingEntity {
    pub fn new(coord: CoordGrid, width: u8, length: u8, lifecycle: EntityLifeCycle) -> Self {
        PathingEntity {
            entity: Entity::new(coord, width, length, lifecycle),
        }
    }
}

#[derive(Copy, Clone)]
pub struct NonPathingEntity {
    entity: Entity,
}

impl NonPathingEntity {
    pub fn new(coord: CoordGrid, width: u8, length: u8, lifecycle: EntityLifeCycle) -> Self {
        NonPathingEntity {
            entity: Entity::new(coord, width, length, lifecycle),
        }
    }
}

impl_entity_behavior_for!(PathingEntity, field: entity);
impl_entity_behavior_for!(NonPathingEntity, field: entity);

impl_entity_behavior_for!(Player, delegate: pathing_entity);
impl_entity_behavior_for!(NPC, delegate: pathing_entity);
impl_entity_behavior_for!(NetworkPlayer, delegate: player.pathing_entity);