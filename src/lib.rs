use crate::entities::query::Query;
use std::any::Any;

use ecs_errors::ECSError;

pub mod ecs_errors;
pub mod entities;
mod resources;

#[derive(Default, Debug)]
pub struct World {
    resources: resources::Resources,
    entities: entities::Entities,
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    /**
    Adds a new resource to the world. The resource can be of any type that implements `Any`.
    Once added, the resource can be retrieved or modified by its type. This function consumes
    the resource and stores it in the world.

    Example:
    ```
    use ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);

    let resource = world.get_resource::<u32>().unwrap();
    assert_eq!(*resource, 10);
    ```
    */
    pub fn add_resource(&mut self, resource: impl Any) {
        self.resources.add(resource);
    }

    /**
    Query for a resource and get a mutable reference to it.
    The type of the resource must be added in advance.

    Example:
    ```
    use ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);

    {
        let resource = world.get_resource_mut::<u32>().unwrap();
        *resource += 1;
    }

    let resource = world.get_resource::<u32>().unwrap();
    assert_eq!(*resource, 11);
    ```
    */
    pub fn get_resource_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    /**
    Retrieves a reference to a resource by its type. If the resource exists, it will be returned as a reference.
    Otherwise, `None` is returned.

    Example:
    ```
    use ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);

    if let Some(resource) = world.get_resource::<u32>() {
        assert_eq!(*resource, 10);
    }
    ```
    */
    pub fn get_resource<T: Any>(&self) -> Option<&T> {
        self.resources.get_ref::<T>()
    }

    /**
    Removes a specific resource from the world by its type. The resource is deleted and can no longer be accessed.

    Example:
    ```
    use ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);
    world.remove_resource::<u32>();

    assert!(world.get_resource::<u32>().is_none());
    ```
    */
    pub fn remove_resource<T: Any>(&mut self) {
        self.resources.remove::<T>();
    }

    /**
    Registers a new component type in the world. This component can later be added to entities.
    The type must implement `Any` and have a static lifetime.

    Example:
    ```
    use ecs::World;
    struct Health(pub u32);
    let mut world = World::new();

    world.register_component::<Health>();

    // Health component can now be used for entities
    ```
    */
    pub fn register_component<T: Any + 'static>(&mut self) {
        self.entities.register_component::<T>();
    }

    /**
    Creates a new entity. The entity is initially empty and can later be populated with components.
    This function returns a mutable reference to the entity system, allowing you to chain component
    additions to the created entity.

    Example:
    ```
    use ecs::World;
    use ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        world.register_component::<Speed>();

        let entity = world.create_entity()
            .with_component(Health(100))?
            .with_component(Speed(15.0))?;

        Ok(())
    }

    // The entity now has Health and Speed components
    ```
    */
    pub fn create_entity(&mut self) -> &mut entities::Entities {
        self.entities.create_entity()
    }

    /**
    Adds a component to an entity by its ID. The component must be registered beforehand.
    This function updates the entity with the provided component data.

    Example:
    ```
    use ecs::World;
    use ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        let entity = world.create_entity().with_component(Health(100))?;

        // Add a new component to the entity with ID 0
        world.add_component_to_entity_by_id(0, Speed(15.0))?;

        Ok(())
    }
    ```
    */
    pub fn add_component_to_entity(
        &mut self,
        entity_id: usize,
        component_data: impl Any,
    ) -> Result<(), ECSError> {
        self.entities
            .add_component_to_entity(entity_id, component_data)
    }

    /**
    Removes an entity by its ID. The entity and its associated components will be removed from the world.
    If the entity does not exist, an error will be returned.

    Example:
    ```
    use ecs::World;
    use ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        let entity = world.create_entity().with_component(Health(100))?;

        // Remove the entity with ID 0
        world.remove_entity_by_id(0)?;

        Ok(())
    }
    ```
    */
    pub fn remove_entity(&mut self, entity_id: usize) -> Result<(), ECSError> {
        self.entities.remove_entity(entity_id)
    }

    /**
    Removes a specific component from an entity by its ID. The component type must be registered
    in advance. The function will attempt to remove the component from the entity and return any errors
    if the component is not registered or if there is an issue.

    Example:
    ```
    use ecs::World;
    use ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        let entity = world.create_entity().with_component(Health(100))?;

        // Remove the Health component from entity 0
        world.remove_component_by_entity_id::<Health>(0)?;

        Ok(())
    }
    ```
    */
    pub fn remove_entity_component<T: Any>(&mut self, entity_id: usize) -> Result<(), ECSError> {
        self.entities.remove_entity_component::<T>(entity_id)
    }

    /**
    Query the entities in the world to retrieve components based on the registered component types.
    You can use this to filter and retrieve entities with specific combinations of components.

    Example:
    ```
    use ecs::World;
    use ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        world.register_component::<Speed>();

        let mut entity = world.create_entity();
        entity.with_component(Health(100))?;
        entity.with_component(Speed(15.0))?;

        let query = world.query();

        Ok(())
    }
    // Perform queries based on your component types
    ```
    */
    pub fn query(&self) -> Query {
        Query::new(&self.entities)
    }
}
