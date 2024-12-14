use std::any::{Any, TypeId};

use crate::ecs_errors::ECSError;

use super::{query_entity::QueryEntity, Component, Entities};

pub type QueryIndexes = Vec<usize>;
pub type QueryComponents = Vec<Vec<Component>>;

#[derive(Debug)]
pub struct Query<'a> {
    map: u32,
    entities: &'a Entities,
    type_ids: Vec<TypeId>,
}

impl<'a> Query<'a> {
    pub fn new(entities: &'a Entities) -> Self {
        Self {
            entities,
            map: 0,
            type_ids: vec![],
        }
    }

    pub fn with_component<T: Any>(&mut self) -> Result<&mut Self, ECSError> {
        let type_id = TypeId::of::<T>();
        match self.entities.get_bitmask(&type_id) {
            Some(bit_mask) => {
                self.map |= bit_mask;
                self.type_ids.push(type_id);
            }
            None => return Err(ECSError::ComponentNotRegistered),
        }
        Ok(self)
    }

    pub fn run(&self) -> (QueryIndexes, QueryComponents) {
        let indexes: Vec<usize> = self
            .entities
            .components_map
            .iter()
            .enumerate()
            .filter_map(
                |(index, entity_map)| match entity_map & self.map == self.map {
                    true => Some(index),
                    false => None,
                },
            )
            .collect();
        let mut result = vec![];

        for type_id in &self.type_ids {
            let entity_components = self.entities.components.get(type_id).unwrap();
            let mut components_to_keep = vec![];
            for index in &indexes {
                components_to_keep.push(entity_components[*index].as_ref().unwrap().clone());
            }
            result.push(components_to_keep);
        }

        (indexes, result)
    }

    pub fn run_entity(&self) -> Vec<QueryEntity> {
        self.entities
            .components_map
            .iter()
            .enumerate()
            .filter_map(|(index, entity_map)| {
                if entity_map & self.map == self.map {
                    Some(QueryEntity::new(index, self.entities))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::entities::query_entity::QueryEntity;

    use super::*;
    use core::f32;
    use std::{
        cell::{Ref, RefMut},
        u32,
    };

    #[test]
    fn query_mask_updating_with_component() -> Result<(), ECSError> {
        let mut entities = Entities::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();

        let mut query = Query::new(&entities);

        query.with_component::<u32>()?.with_component::<f32>()?;

        assert_eq!(query.map, 3);
        assert_eq!(TypeId::of::<u32>(), query.type_ids[0]);
        assert_eq!(TypeId::of::<f32>(), query.type_ids[1]);

        Ok(())
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn run_query() -> Result<(), ECSError> {
        let mut entities = Entities::default();

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
        query.with_component::<u32>()?.with_component::<f32>()?;

        let query_result = query.run();
        let u32s = &query_result.1[0];
        let f32s = &query_result.1[1];
        let indexes = &query_result.0;

        assert!(u32s.len() == f32s.len() && u32s.len() == indexes.len());
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

        assert_eq!(indexes[0], 0);
        assert_eq!(indexes[1], 3);

        Ok(())
    }

    #[test]
    fn run_query_with_no_components() -> Result<(), ECSError> {
        let mut entities = Entities::default();

        entities.register_component::<u32>();
        entities.create_entity().with_component(10_u32)?;
        entities.create_entity();

        let mut query = Query::new(&entities);

        query.with_component::<u32>()?;

        let query_result = query.run();
        let u32s = &query_result.1[0];

        assert_eq!(u32s.len(), 1);

        Ok(())
    }

    #[test]
    fn query_after_deleting_entity() -> Result<(), ECSError> {
        let mut entities = Entities::default();

        entities.register_component::<u32>();
        entities.create_entity().with_component(10_u32)?;
        entities.create_entity().with_component(20_u32)?;
        entities.remove_entity(1)?;

        let (query_indexes, query_results) = Query::new(&entities).with_component::<u32>()?.run();

        assert_eq!(query_indexes.len(), query_results.len());
        assert_eq!(query_results[0].len(), 1);
        assert_eq!(query_indexes[0], 0);

        let borrowed_first_u32 = query_results[0][0].borrow();
        let first_u32 = borrowed_first_u32.downcast_ref::<u32>().unwrap();

        assert_eq!(*first_u32, 10);

        Ok(())
    }

    #[test]
    fn query_for_entity_ref() -> Result<(), ECSError> {
        let mut entities = Entities::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();
        entities.create_entity().with_component(100_u32)?;
        entities.create_entity().with_component(10.0_f32)?;

        let mut query = Query::new(&entities);
        let entities: Vec<QueryEntity> = query.with_component::<u32>()?.run_entity();

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
        let mut entities = Entities::default();

        entities.register_component::<u32>();
        entities.register_component::<f32>();
        entities.create_entity().with_component(100_u32)?;
        entities.create_entity().with_component(10.0_f32)?;

        let mut query = Query::new(&entities);
        let entities: Vec<QueryEntity> = query.with_component::<u32>()?.run_entity();

        assert_eq!(entities.len(), 1);

        for mut entity in entities {
            assert_eq!(entity.id, 0);
            let mut health: RefMut<u32> = entity.get_component_mut::<u32>()?;
            assert_eq!(*health, 100);
            *health += 1;
        }

        let entities: Vec<QueryEntity> = query.with_component::<u32>()?.run_entity();

        for entity in entities {
            let health: Ref<u32> = entity.get_component::<u32>()?;
            assert_eq!(*health, 101);
        }

        Ok(())
    }
}
