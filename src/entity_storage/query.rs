use std::any::{Any, TypeId};

use super::{query_entity::QueryEntity, Component, EntityStorage};
use crate::ecs_errors::ECSError;

pub type MatchedEntityIds = Vec<usize>;
pub type MatchedComponents = Vec<Vec<Component>>;

pub struct QueryResult {
    pub entity_ids: MatchedEntityIds,
    pub components: MatchedComponents,
}

#[derive(Debug)]
pub struct Query<'a> {
    filter_mask: u32,
    entity_storage: &'a EntityStorage,
    component_type_ids: Vec<TypeId>,
}

impl<'a> Query<'a> {
    pub fn new(entity_storage: &'a EntityStorage) -> Self {
        Self {
            entity_storage,
            filter_mask: 0,
            component_type_ids: vec![],
        }
    }

    pub fn with_component_filter<T: Any>(&mut self) -> Result<&mut Self, ECSError> {
        let component_type_id = TypeId::of::<T>();

        match self.entity_storage.get_bitmask(&component_type_id) {
            Some(bitmask) => {
                self.filter_mask |= bitmask;
                self.component_type_ids.push(component_type_id);
            }
            None => return Err(ECSError::ComponentNotRegistered),
        }
        Ok(self)
    }

    pub fn run(&self) -> QueryResult {
        let matched_entity_ids: Vec<usize> = self
            .entity_storage
            .entity_component_bitmasks
            .iter()
            .enumerate()
            .filter_map(|(index, entity_map)| {
                match entity_map & self.filter_mask == self.filter_mask {
                    true => Some(index),
                    false => None,
                }
            })
            .collect();

        let mut matched_components = vec![];

        for type_id in &self.component_type_ids {
            let entity_components = self.entity_storage.components.get(type_id).unwrap();
            let mut components_to_keep = vec![];

            for index in &matched_entity_ids {
                components_to_keep.push(entity_components[*index].as_ref().unwrap().clone());
            }

            matched_components.push(components_to_keep);
        }

        QueryResult {
            entity_ids: matched_entity_ids,
            components: matched_components,
        }
    }

    pub fn get_entities(&self) -> Vec<QueryEntity> {
        self.entity_storage
            .entity_component_bitmasks
            .iter()
            .enumerate()
            .filter_map(|(entity_id, entity_map)| {
                if entity_map & self.filter_mask == self.filter_mask {
                    Some(QueryEntity::new(entity_id, self.entity_storage))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::entity_storage::query_entity::QueryEntity;

    use super::*;
    use core::f32;
    use std::cell::{Ref, RefMut};

    #[test]
    fn query_mask_updating_with_component() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();

        let mut query = Query::new(&entities);

        query
            .with_component_filter::<u32>()?
            .with_component_filter::<f32>()?;

        assert_eq!(query.filter_mask, 3);
        assert_eq!(TypeId::of::<u32>(), query.component_type_ids[0]);
        assert_eq!(TypeId::of::<f32>(), query.component_type_ids[1]);
        Ok(())
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn run_query() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();

        entities
            .create_entity()
            .with_component(10_u32)?
            .with_component(20.0_f32)?;

        entities.create_entity().with_component(5_u32)?;
        entities.create_entity().with_component(50.0_f32)?;
        entities
            .create_entity()
            .with_component(15_u32)?
            .with_component(25.0_f32)?;

        let mut query = Query::new(&entities);
        query
            .with_component_filter::<u32>()?
            .with_component_filter::<f32>()?;

        let query_result = query.run();
        let u32s = &query_result.components[0];
        let f32s = &query_result.components[1];
        let entity_ids = &query_result.entity_ids;

        assert!(u32s.len() == f32s.len() && u32s.len() == entity_ids.len());
        assert_eq!(u32s.len(), 2);

        let borrowed_first_u32 = u32s[0].borrow();
        let first_u32 = borrowed_first_u32.downcast_ref::<u32>().unwrap();
        assert_eq!(*first_u32, 10);

        let borrowed_first_f32 = f32s[0].borrow();
        let first_f32 = borrowed_first_f32.downcast_ref::<f32>().unwrap();
        assert_eq!(*first_f32, 20.0);

        let borrowed_second_u32 = u32s[1].borrow();
        let second_u32 = borrowed_second_u32.downcast_ref::<u32>().unwrap();
        assert_eq!(*second_u32, 15);

        let borrowed_second_f32 = f32s[1].borrow();
        let second_f32 = borrowed_second_f32.downcast_ref::<f32>().unwrap();

        assert_eq!(*second_f32, 25.0);
        assert_eq!(entity_ids[0], 0);
        assert_eq!(entity_ids[1], 3);
        Ok(())
    }

    #[test]
    fn run_query_with_no_components() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.create_entity().with_component(10_u32)?;
        entities.create_entity();

        let mut query = Query::new(&entities);

        query.with_component_filter::<u32>()?;

        let query_result = query.run();
        let u32s = &query_result.components[0];

        assert_eq!(u32s.len(), 1);
        Ok(())
    }

    #[test]
    fn query_after_deleting_entity() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.create_entity().with_component(10_u32)?;
        entities.create_entity().with_component(20_u32)?;
        entities.remove_entity(1)?;

        let result = Query::new(&entities).with_component_filter::<u32>()?.run();
        let entity_ids = result.entity_ids;
        let components = result.components;

        assert_eq!(entity_ids.len(), components.len());
        assert_eq!(components[0].len(), 1);
        assert_eq!(entity_ids[0], 0);

        let borrowed_first_u32 = components[0][0].borrow();
        let first_u32 = borrowed_first_u32.downcast_ref::<u32>().unwrap();

        assert_eq!(*first_u32, 10);
        Ok(())
    }

    #[test]
    fn query_for_entity_ref() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();
        entities.create_entity().with_component(100_u32)?;
        entities.create_entity().with_component(10.0_f32)?;

        let mut query = Query::new(&entities);
        let entities: Vec<QueryEntity> = query.with_component_filter::<u32>()?.get_entities();

        assert_eq!(entities.len(), 1);

        for entity in entities {
            assert_eq!(entity.id, 0);
            let health: Ref<u32> = entity.get_component::<u32>()?;
            assert_eq!(*health, 100);
        }

        Ok(())
    }

    #[test]
    fn query_for_entity_mut() -> Result<(), ECSError> {
        let mut entities = EntityStorage::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();
        entities.create_entity().with_component(100_u32)?;
        entities.create_entity().with_component(10.0_f32)?;

        let mut query = Query::new(&entities);
        let entities: Vec<QueryEntity> = query.with_component_filter::<u32>()?.get_entities();

        assert_eq!(entities.len(), 1);

        for mut entity in entities {
            assert_eq!(entity.id, 0);
            let mut health: RefMut<u32> = entity.get_component_mut::<u32>()?;
            assert_eq!(*health, 100);
            *health += 1;
        }

        let entities: Vec<QueryEntity> = query.with_component_filter::<u32>()?.get_entities();

        for entity in entities {
            let health: Ref<u32> = entity.get_component::<u32>()?;
            assert_eq!(*health, 101);
        }

        Ok(())
    }
}
