use std::collections::VecDeque;
use crate::entity::entity::EntityBehavior;
use crate::entity::npc::NPC;
use crate::entity::player::Player;

pub struct EntityList<T: EntityBehavior> {
    entities: Vec<Option<T>>,
    id_to_index: Vec<usize>,
    free_indices: VecDeque<usize>,
    index_padding: usize,
    last_used_index: usize,
}

impl<T: EntityBehavior> EntityList<T> {
    pub fn new(size: usize, index_padding: usize) -> Self {
        let id_to_index = vec![usize::MAX; size];

        let mut entities = Vec::with_capacity(size);
        for _ in 0..size {
            entities.push(None);
        }

        let mut free_indices = VecDeque::with_capacity(size);
        for i in (0..size).rev() {
            free_indices.push_back(i);
        }

        EntityList {
            entities,
            id_to_index,
            free_indices,
            index_padding,
            last_used_index: 0,
        }
    }

    pub fn next(&self, _priority: bool, start: Option<usize>) -> Result<usize, &'static str> {
        let start = start.unwrap_or(self.last_used_index + 1);

        if let Some(index) = (start..self.id_to_index.len()).find(|&index| self.id_to_index[index] == usize::MAX) {
            return Ok(index);
        }

        let end = start.min(self.id_to_index.len());
        if let Some(index) = (self.index_padding..end).find(|&index| self.id_to_index[index] == usize::MAX) {
            return Ok(index);
        }

        Err("No space for new entities")
    }

    pub fn count(&self) -> usize {
        self.entities.len() - self.free_indices.len()
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        if id >= self.id_to_index.len() {
            return None;
        }

        let index = self.id_to_index[id];
        if index == usize::MAX {
            None
        } else {
            self.entities[index].as_ref()
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        if id >= self.id_to_index.len() {
            return None;
        }

        let index = self.id_to_index[id];
        if index == usize::MAX {
            None
        } else {
            self.entities[index].as_mut()
        }
    }

    pub fn set(&mut self, id: usize, entity: T) -> Result<(), &'static str> {
        if id >= self.id_to_index.len() {
            return Err("ID out of bounds");
        }

        if self.id_to_index[id] != usize::MAX {
            return Err("ID already in use");
        }

        let index = match self.free_indices.pop_front() {
            Some(index) => index,
            None => return Err("Cannot find available entities slot"),
        };

        self.entities[index] = Some(entity);
        self.id_to_index[id] = index;
        self.last_used_index = id;

        Ok(())
    }

    pub fn remove(&mut self, id: usize) {
        if id < self.id_to_index.len() {
            let index = self.id_to_index[id];
            if index != usize::MAX {
                self.id_to_index[id] = usize::MAX;
                self.entities[index] = None;
                self.free_indices.push_back(index);
            }
        }
    }

    pub fn reset(&mut self) {
        for entity in &mut self.entities {
            *entity = None;
        }

        for id in &mut self.id_to_index {
            *id = usize::MAX;
        }

        self.free_indices.clear();
        for i in (0..self.entities.len()).rev() {
            self.free_indices.push_back(i);
        }

        self.last_used_index = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        (0..self.id_to_index.len())
            .filter_map(move |id| {
                let idx = self.id_to_index[id];
                if idx != usize::MAX {
                    if let Some(entity) = &self.entities[idx] {
                        return Some((id, entity));
                    }
                }
                None
            })
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        for id in 0..self.id_to_index.len() {
            let index = self.id_to_index[id];
            if index == usize::MAX {
                continue;
            }
            
            if let Some(entity) = &self.entities[index] {
                f(entity);
            }
        }
    }

    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        for id in 0..self.id_to_index.len() {
            let index = self.id_to_index[id];
            if index != usize::MAX {
                if let Some(entity) = &mut self.entities[index] {
                    f(entity);
                }
            }
        }
    }
}

pub struct PlayerList {
    list: EntityList<Player>,
}

impl PlayerList {
    pub fn new(size: usize) -> Self {
        PlayerList {
            list: EntityList::new(size, 1),
        }
    }

    pub fn next(&self, priority: bool, start: Option<usize>) -> Result<usize, &'static str> {
        let start_index = start.unwrap_or(self.list.last_used_index + 1);

        if priority {
            for offset in 0..100 {
                let index = start_index + offset;
                if index < self.list.id_to_index.len() && self.list.id_to_index[index] == usize::MAX {
                    return Ok(index);
                }
            }
        }

        self.list.next(false, Some(start_index))
    }

    pub fn count(&self) -> usize {
        self.list.count()
    }

    pub fn get(&self, id: usize) -> Option<&Player> {
        self.list.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Player> {
        self.list.get_mut(id)
    }

    pub fn set(&mut self, id: usize, entity: Player) -> Result<(), &'static str> {
        self.list.set(id, entity)
    }

    pub fn remove(&mut self, id: usize) {
        self.list.remove(id)
    }

    pub fn reset(&mut self) {
        self.list.reset()
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&Player)
    {
        self.list.for_each(f)
    }

    pub fn for_each_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut Player)
    {
        self.list.for_each_mut(f)
    }
}

pub struct NPCList {
    list: EntityList<NPC>,
}

impl NPCList {
    pub fn new(size: usize) -> Self {
        NPCList {
            list: EntityList::new(size, 1),
        }
    }

    pub fn count(&self) -> usize {
        self.list.count()
    }

    pub fn get(&self, id: usize) -> Option<&NPC> {
        self.list.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut NPC> {
        self.list.get_mut(id)
    }

    pub fn set(&mut self, id: usize, entity: NPC) -> Result<(), &'static str> {
        self.list.set(id, entity)
    }

    pub fn remove(&mut self, id: usize) {
        self.list.remove(id)
    }

    pub fn reset(&mut self) {
        self.list.reset()
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&NPC)
    {
        self.list.for_each(f)
    }

    pub fn for_each_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut NPC)
    {
        self.list.for_each_mut(f)
    }
}