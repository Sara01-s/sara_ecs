use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use super::EntityStorage;
use crate::ecs_errors::ECSError;

type ExtractedComponents<'a> = &'a Vec<Option<Rc<RefCell<dyn Any>>>>;

pub struct QueryEntity<'a> {
    pub id: usize,
    entities: &'a EntityStorage,
}

impl<'a> QueryEntity<'a> {
    pub fn new(id: usize, entities: &'a EntityStorage) -> Self {
        Self { id, entities }
    }

    pub fn get_component<T: Any>(&self) -> Result<Ref<T>, ECSError> {
        let components = self.extract_components::<T>()?;
        let borrowed_component = components[self.id]
            .as_ref()
            .ok_or(ECSError::ComponentDoesNotExist)?
            .borrow();

        Ok(Ref::map(borrowed_component, |any| {
            any.downcast_ref::<T>().unwrap()
        }))
    }

    pub fn get_component_mut<T: Any>(&mut self) -> Result<RefMut<T>, ECSError> {
        let components = self.extract_components::<T>()?;
        let borrowed_component = components[self.id]
            .as_ref()
            .ok_or(ECSError::ComponentDoesNotExist)?
            .borrow_mut();

        Ok(RefMut::map(borrowed_component, |any| {
            any.downcast_mut::<T>().unwrap()
        }))
    }

    fn extract_components<T: Any>(&self) -> Result<ExtractedComponents, ECSError> {
        let component_type_id = TypeId::of::<T>();
        let components = self
            .entities
            .components
            .get(&component_type_id)
            .ok_or(ECSError::ComponentNotRegistered);

        components
    }
}
