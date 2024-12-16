use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

use crate::ecs_errors::ECSError;

#[derive(Default, Debug)]
pub struct ResourceStorage {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceStorage {
    pub fn insert<T: Any>(&mut self, data: T) -> Result<(), ECSError> {
        let type_id = TypeId::of::<T>();

        if self.data.contains_key(&type_id) {
            return Err(ECSError::ResourceAlreadyRegistered);
        }

        self.data.insert(type_id, Box::new(data));
        Ok(())
    }

    pub fn replace<T: Any>(&mut self, data: T) {
        let type_id = TypeId::of::<T>();
        self.data.insert(type_id, Box::new(data));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.data.get(&type_id)?.downcast_ref()
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.data.get_mut(&type_id)?.downcast_mut()
    }

    pub fn remove<T: Any>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        self.data.remove(&type_id).is_some()
    }

    #[must_use]
    pub fn contains<T: Any>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.data.contains_key(&type_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct WorldWidth(pub f32); // Test resource.

    #[test]
    fn insert_resource() {
        let mut resources = ResourceStorage::default();

        assert!(resources.insert(WorldWidth(100.0)).is_ok());
        assert!(resources.contains::<WorldWidth>());
    }

    #[test]
    fn insert_duplicate_resource_fails() {
        let mut resources = ResourceStorage::default();
        resources.insert(WorldWidth(100.0)).unwrap();

        let result = resources.insert(WorldWidth(200.0));

        assert!(result.is_err());
    }

    #[test]
    fn replace_resource() {
        let mut resources = ResourceStorage::default();

        resources.insert(WorldWidth(100.0)).unwrap();
        resources.replace(WorldWidth(200.0));

        let world_width = resources.get::<WorldWidth>().unwrap();

        assert_eq!(world_width.0, 200.0);
    }

    #[test]
    fn get_resource() {
        let mut resources = ResourceStorage::default();
        resources.insert(WorldWidth(100.0)).unwrap();

        if let Some(world_width) = resources.get::<WorldWidth>() {
            assert_eq!(world_width.0, 100.0);
        } else {
            panic!("Resource not found");
        }
    }

    #[test]
    fn get_resource_mut() {
        let mut resources = ResourceStorage::default();
        resources.insert(WorldWidth(100.0)).unwrap();

        {
            let world_width = resources.get_mut::<WorldWidth>().unwrap();
            world_width.0 += 1.0;
        }

        let world_width = resources.get::<WorldWidth>().unwrap();

        assert_eq!(world_width.0, 101.0);
    }

    #[test]
    fn remove_resource() {
        let mut resources = ResourceStorage::default();
        resources.insert(WorldWidth(100.0)).unwrap();

        assert!(resources.remove::<WorldWidth>());
        assert!(!resources.contains::<WorldWidth>());
    }

    #[test]
    fn contains_resource() {
        let mut resources = ResourceStorage::default();
        resources.insert(WorldWidth(100.0)).unwrap();

        assert!(resources.contains::<WorldWidth>());

        resources.remove::<WorldWidth>();

        assert!(!resources.contains::<WorldWidth>());
    }
}
