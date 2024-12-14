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
pub struct Entities {
    components: Components,
    component_bit_masks: HashMap<TypeId, u32>,
    components_map: Vec<u32>,
    inserting_into_index: usize,
}

impl Entities {
    pub fn register_component<T: Any + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, vec![]);
        self.component_bit_masks
            .insert(type_id, 1 << self.component_bit_masks.len());
    }

    pub fn create_entity(&mut self) -> &mut Self {
        if let Some((index, _)) = self
            .components_map
            .iter()
            .enumerate()
            .find(|(_index, mask)| **mask == 0)
        {
            self.inserting_into_index = index;
        } else {
            self.components
                .iter_mut()
                .for_each(|(_key, components)| components.push(None));

            self.components_map.push(0);
            self.inserting_into_index = self.components_map.len() - 1;
        }

        self
    }

    pub fn with_component(&mut self, data: impl Any) -> Result<&mut Self, ECSError> {
        let type_id = data.type_id();
        let index = self.inserting_into_index;
        if let Some(components) = self.components.get_mut(&type_id) {
            let component = components
                .get_mut(index)
                .ok_or(ECSError::CreateComponentNeverCalled)?;
            *component = Some(Rc::new(RefCell::new(data)));

            let bitmask = self.component_bit_masks.get(&type_id).unwrap();
            self.components_map[index] |= *bitmask;
        } else {
            return Err(ECSError::ComponentNotRegistered.into());
        }
        Ok(self)
    }

    pub fn get_bitmask(&self, type_id: &TypeId) -> Option<u32> {
        self.component_bit_masks.get(type_id).copied()
    }

    pub fn remove_entity_component<T: Any>(&mut self, index: usize) -> Result<(), ECSError> {
        let type_id = TypeId::of::<T>();
        let mask = if let Some(mask) = self.component_bit_masks.get(&type_id) {
            mask
        } else {
            return Err(ECSError::ComponentNotRegistered.into());
        };

        if self.has_component(index, *mask) {
            self.components_map[index] ^= *mask;
        }

        Ok(())
    }

    pub fn add_component_to_entity(
        &mut self,
        index: usize,
        data: impl Any,
    ) -> Result<(), ECSError> {
        let type_id = data.type_id();
        let mask = if let Some(mask) = self.component_bit_masks.get(&type_id) {
            mask
        } else {
            return Err(ECSError::ComponentNotRegistered.into());
        };
        self.components_map[index] |= *mask;

        let components = self.components.get_mut(&type_id).unwrap();
        components[index] = Some(Rc::new(RefCell::new(data)));

        Ok(())
    }

    pub fn remove_entity(&mut self, index: usize) -> Result<(), ECSError> {
        if let Some(map) = self.components_map.get_mut(index) {
            *map = 0;
        } else {
            return Err(ECSError::EntityDoesNotExist.into());
        }
        Ok(())
    }

    fn has_component(&self, index: usize, mask: u32) -> bool {
        self.components_map[index] & mask == mask
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::ecs_errors::ECSError;

    use super::*;

    struct Health(pub u32);
    struct Speed(pub u32);

    #[test]
    fn register_an_entity() {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        let type_id = TypeId::of::<Health>();
        let health_components = entities.components.get(&type_id).unwrap();
        assert_eq!(health_components.len(), 0);
    }

    #[test]
    fn bitmask_updated_when_registering_entities() {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        let type_id = TypeId::of::<Health>();
        let mask = entities.component_bit_masks.get(&type_id).unwrap();
        assert_eq!(*mask, 1);

        entities.register_component::<Speed>();
        let type_id = TypeId::of::<Speed>();
        let mask = entities.component_bit_masks.get(&type_id).unwrap();
        assert_eq!(*mask, 2);
    }

    #[test]
    fn create_entity() {
        let mut entities = Entities::default();
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
        let mut entities = Entities::default();
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
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(15))?;
        let entity_map = entities.components_map[0];
        assert_eq!(entity_map, 3);

        entities.create_entity().with_component(Speed(15))?;
        let entity_map = entities.components_map[1];
        assert_eq!(entity_map, 2);
        Ok(())
    }

    #[test]
    fn remove_component_by_entity_id() -> Result<(), ECSError> {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities
            .create_entity()
            .with_component(Health(100))?
            .with_component(Speed(50))?;

        entities.remove_entity_component::<Health>(0)?;

        assert_eq!(entities.components_map[0], 2);
        Ok(())
    }

    #[test]
    fn add_component_to_entity_by_id() -> Result<(), ECSError> {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.register_component::<Speed>();
        entities.create_entity().with_component(Health(100))?;

        entities.add_component_to_entity(0, Speed(50))?;

        assert_eq!(entities.components_map[0], 3);

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
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.create_entity().with_component(Health(100))?;
        entities.remove_entity(0)?;
        assert_eq!(entities.components_map[0], 0);
        Ok(())
    }

    #[test]
    fn created_entities_are_inserted_into_deleted_entities_columns() -> Result<(), ECSError> {
        let mut entities = Entities::default();
        entities.register_component::<Health>();
        entities.create_entity().with_component(Health(100))?;
        entities.create_entity().with_component(Health(50))?;
        entities.remove_entity(0)?;
        entities.create_entity().with_component(Health(25))?;

        assert_eq!(entities.components_map[0], 1);

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
        let mut entities = Entities::default();
        entities.register_component::<u32>();
        entities.register_component::<f32>();
        entities
            .create_entity()
            .with_component(100_u32)?
            .with_component(50.0_f32)?;
        entities.remove_entity_component::<u32>(0)?;
        entities.remove_entity_component::<u32>(0)?;
        assert_eq!(entities.components_map[0], 2);
        Ok(())
    }

    /*
       Brendon Stanton
           When you create your second component, inserting_into_index never changes
           from 0.  So when you get to with_component, Health(50) overwrites Health(100)
           and column 1 in your table never actually gets used.
    */
    #[test]
    fn inserting_into_index_should_change_when_adding_components() -> Result<(), ECSError> {
        let mut entities = Entities::default();
        entities.register_component::<f32>();
        entities.register_component::<u32>();

        // Inserting an entity with 2 components to make sure that inserting_into_index is correct
        let creating_entity = entities.create_entity();
        assert_eq!(creating_entity.inserting_into_index, 0);
        creating_entity
            .with_component(100.0_f32)?
            .with_component(10_u32)?;
        assert_eq!(entities.inserting_into_index, 0);

        // Inserting another entity with 2 components to make sure that the inserting_into_index is now 1
        let creating_entity = entities.create_entity();
        assert_eq!(creating_entity.inserting_into_index, 1);
        creating_entity
            .with_component(110.0_f32)?
            .with_component(20_u32)?;
        assert_eq!(entities.inserting_into_index, 1);

        // delete the first entity, and re-create to make sure that inserting_into_index is back
        // to 0 again
        entities.remove_entity(0)?;
        let creating_entity = entities.create_entity();
        assert_eq!(creating_entity.inserting_into_index, 0);
        creating_entity
            .with_component(100.0_f32)?
            .with_component(10_u32)?;
        assert_eq!(entities.inserting_into_index, 0);
        Ok(())
    }
}
