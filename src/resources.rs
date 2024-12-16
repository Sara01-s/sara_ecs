use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Resources {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    pub fn add(&mut self, data: impl Any) {
        let type_id = data.type_id();
        self.data.insert(type_id, Box::new(data));
    }

    pub fn get_ref<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        match self.data.get(&type_id) {
            Some(data) => data.downcast_ref(),
            None => None,
        }
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();

        match self.data.get_mut(&type_id) {
            Some(data) => data.downcast_mut(),
            None => None,
        }
    }

    pub fn remove<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.data.remove(&type_id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct WorldWidth(pub f32);

    #[test]
    fn add_resource() {
        let resources = get_test_resources();
        let stored_resource = resources.data.get(&TypeId::of::<WorldWidth>()).unwrap();
        let extracted_world_width = stored_resource.downcast_ref::<WorldWidth>().unwrap();

        assert_eq!(extracted_world_width.0, 100.0);
    }

    #[test]
    fn get_resource() {
        let resources = get_test_resources();

        if let Some(extracted_world_width) = resources.get_ref::<WorldWidth>() {
            assert_eq!(extracted_world_width.0, 100.0);
        }
    }

    #[test]
    fn get_resource_mut() {
        let mut resources = get_test_resources();

        {
            let world_width = resources.get_mut::<WorldWidth>().unwrap();
            world_width.0 += 1.0;
        }

        let world_width = resources.get_ref::<WorldWidth>().unwrap();
        assert_eq!(world_width.0, 101.0);
    }

    #[test]
    fn remove_resource() {
        let mut resources = get_test_resources();
        resources.remove::<WorldWidth>();

        let world_width_type_id = TypeId::of::<WorldWidth>();
        let is_resource_deleted = !resources.data.contains_key(&world_width_type_id);

        assert!(is_resource_deleted);
    }

    fn get_test_resources() -> Resources {
        let mut resources = Resources::default();
        let world_width = WorldWidth(100.0);

        resources.add(world_width);
        resources
    }
}
