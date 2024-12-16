pub mod query;
pub mod query_entity;

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    vec,
};

use crate::ecs_errors::ECSError;

pub type Component = Rc<RefCell<dyn Any>>;
pub type Components = HashMap<TypeId, Vec<Option<Component>>>;

#[derive(Debug, Default)]
pub struct EntityStorage {
    components: Components,
    component_bitmasks: HashMap<TypeId, u32>,
    entity_component_bitmasks: Vec<u32>,
    next_free_entity_id: usize,
}

impl EntityStorage {
    pub fn register_component<T: Any + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();

        self.components.insert(type_id, vec![]);
        self.component_bitmasks
            .insert(type_id, 1 << self.component_bitmasks.len());
    }

    pub fn create_entity(&mut self) -> &mut Self {
        if let Some((index, _)) = self
            .entity_component_bitmasks
            .iter()
            .enumerate()
            .find(|(_index, mask)| **mask == 0)
        {
            self.next_free_entity_id = index;
        } else {
            self.components
                .iter_mut()
                .for_each(|(_key, components)| components.push(None));

            self.entity_component_bitmasks.push(0);
            self.next_free_entity_id = self.entity_component_bitmasks.len() - 1;
        }

        self
    }

    pub fn with_component(&mut self, data: impl Any) -> Result<&mut Self, ECSError> {
        let type_id = data.type_id();
        let index = self.next_free_entity_id;

        if let Some(components) = self.components.get_mut(&type_id) {
            let component = components
                .get_mut(index)
                .ok_or(ECSError::CreateComponentNeverCalled)?;
            *component = Some(Rc::new(RefCell::new(data)));

            let bitmask = self.component_bitmasks.get(&type_id).unwrap();
            self.entity_component_bitmasks[index] |= *bitmask;
        } else {
            return Err(ECSError::ComponentNotRegistered.into());
        }
        Ok(self)
    }

    pub fn get_bitmask(&self, type_id: &TypeId) -> Option<u32> {
        self.component_bitmasks.get(type_id).copied()
    }

    pub fn remove_entity_component<T: Any>(&mut self, index: usize) -> Result<(), ECSError> {
        let type_id = TypeId::of::<T>();

        let mask = if let Some(mask) = self.component_bitmasks.get(&type_id) {
            mask
        } else {
            return Err(ECSError::ComponentNotRegistered.into());
        };

        if self.has_component(index, *mask) {
            self.entity_component_bitmasks[index] ^= *mask;
        }

        Ok(())
    }

    pub fn add_component_to_entity(
        &mut self,
        index: usize,
        data: impl Any,
    ) -> Result<(), ECSError> {
        let type_id = data.type_id();
        let mask = if let Some(mask) = self.component_bitmasks.get(&type_id) {
            mask
        } else {
            return Err(ECSError::ComponentNotRegistered.into());
        };
        self.entity_component_bitmasks[index] |= *mask;

        let components = self.components.get_mut(&type_id).unwrap();
        components[index] = Some(Rc::new(RefCell::new(data)));

        Ok(())
    }

    pub fn remove_entity(&mut self, index: usize) -> Result<(), ECSError> {
        match self.entity_component_bitmasks.get_mut(index) {
            Some(map) => *map = 0,
            None => return Err(ECSError::EntityDoesNotExist.into()),
        }

        Ok(())
    }

    fn has_component(&self, index: usize, mask: u32) -> bool {
        self.entity_component_bitmasks[index] & mask == mask
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ecs_errors::ECSError;
    use std::any::TypeId;

    struct Health(pub u32);
    struct Speed(pub u32);

    #[test]
    fn register_an_entity() {
        let mut entities = EntityStorage::default();
        entities.register_component::<Health>();

        let type_id = TypeId::of::<Health>();
        let health_components = entities.components.get(&type_id).unwrap();

        assert_eq!(health_components.len(), 0);
    }

    #[test]
    fn bitmask_updated_when_registering_entities() {
        let mut entities = EntityStorage::default();
        entities.register_component::<Health>();

        let type_id = TypeId::of::<Health>();
        let mask = entities.component_bitmasks.get(&type_id).unwrap();

        assert_eq!(*mask, 1);

        entities.register_component::<Speed>();

        let type_id = TypeId::of::<Speed>();
        let mask = entities.component_bitmasks.get(&type_id).unwrap();

        assert_eq!(*mask, 2);
    }

    #[test]
    fn create_entity() {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities.create_entity();

        let health = entities.components.get(&TypeId::of::<Health>()).unwrap();
        let speed = entities.components.get(&TypeId::of::<Speed>()).unwrap();

        assert!(health.len() == speed.len() && health.len() == 1);
        assert!(health[0].is_none() && speed[0].is_none());
    }

    #[test]
    fn with_component() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(15))?;

        let first_health = &entities.components.get(&TypeId::of::<Health>()).unwrap()[0];
        let wrapped_health = first_health.as_ref().unwrap();
        let borrowed_health = wrapped_health.borrow();
        let health = borrowed_health.downcast_ref::<Health>().unwrap();

        assert_eq!(health.0, 100);
        Ok(())
    }

    #[test]
    fn map_is_updated_when_creating_entities() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(15))?;

        let entity_map = entities.entity_component_bitmasks[0];

        assert_eq!(entity_map, 3);
        entities.create_entity().with_component(Speed(15))?;

        let entity_map = entities.entity_component_bitmasks[1];
        assert_eq!(entity_map, 2);
        Ok(())
    }

    #[test]
    fn remove_component_by_entity_id() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(50))?;

        entities.remove_entity_component::<Health>(0)?;

        assert_eq!(entities.entity_component_bitmasks[0], 2);
        Ok(())
    }

    #[test]
    fn add_component_to_entity_by_id() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities.create_entity().with_component(Health(100))?;
        entities.add_component_to_entity(0, Speed(50))?;

        assert_eq!(entities.entity_component_bitmasks[0], 3);

        let speed_type_id = TypeId::of::<Speed>();
        let wrapped_speeds = entities.components.get(&speed_type_id).unwrap();
        let wrapped_speed = wrapped_speeds[0].as_ref().unwrap();
        let borrowed_speed = wrapped_speed.borrow();
        let speed = borrowed_speed.downcast_ref::<Speed>().unwrap();

        assert_eq!(speed.0, 50);
        Ok(())
    }

    #[test]
    fn remove_entity_by_id() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.create_entity().with_component(Health(100))?;
        entities.remove_entity(0)?;

        assert_eq!(entities.entity_component_bitmasks[0], 0);
        Ok(())
    }

    #[test]
    fn created_entities_are_inserted_into_deleted_entities_columns() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<Health>();
        entities.create_entity().with_component(Health(100))?;
        entities.create_entity().with_component(Health(50))?;
        entities.remove_entity(0)?;
        entities.create_entity().with_component(Health(25))?;

        assert_eq!(entities.entity_component_bitmasks[0], 1);

        let type_id = TypeId::of::<Health>();
        let borrowed_health = &entities.components.get(&type_id).unwrap()[0]
            .as_ref()
            .unwrap()
            .borrow();
        let health = borrowed_health.downcast_ref::<Health>().unwrap();

        assert_eq!(health.0, 25);

        Ok(())
    }

    #[test]
    fn should_not_add_component_back_after_deleting_twice() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();
        entities
            .create_entity()
            .with_component(100_u32)?
            .with_component(50.0_f32)?;
        entities.remove_entity_component::<u32>(0)?;
        entities.remove_entity_component::<u32>(0)?;

        assert_eq!(entities.entity_component_bitmasks[0], 2);

        Ok(())
    }

    #[test]
    fn inserting_into_index_should_change_when_adding_components() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<f32>();
        entities.register_component::<u32>();

        // Inserting an entity with 2 components to make sure that inserting_into_index is correct
        let creating_entity = entities.create_entity();

        assert_eq!(creating_entity.next_free_entity_id, 0);
        creating_entity
            .with_component(100.0_f32)?
            .with_component(10_u32)?;
        assert_eq!(entities.next_free_entity_id, 0);

        // Inserting another entity with 2 components to make sure that the inserting_into_index is now 1
        let creating_entity = entities.create_entity();
        assert_eq!(creating_entity.next_free_entity_id, 1);
        creating_entity
            .with_component(110.0_f32)?
            .with_component(20_u32)?;
        assert_eq!(entities.next_free_entity_id, 1);

        // delete the first entity, and re-create to make sure that inserting_into_index is back
        // to 0 again
        entities.remove_entity(0)?;
        let creating_entity = entities.create_entity();

        assert_eq!(creating_entity.next_free_entity_id, 0);

        creating_entity
            .with_component(100.0_f32)?
            .with_component(10_u32)?;

        assert_eq!(entities.next_free_entity_id, 0);
        Ok(())
    }
}
