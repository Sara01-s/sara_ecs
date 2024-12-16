use std::any::Any;

use ecs_errors::ECSError;
use entity_storage::query::Query;

pub mod ecs_errors;
mod entity_storage;
mod resource_storage;

#[derive(Default, Debug)]
pub struct World {
    resource_storage: resource_storage::ResourceStorage,
    entitiy_storage: entity_storage::EntityStorage,
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
    use sara_ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);

    let resource = world.get_resource::<u32>().unwrap();
    assert_eq!(*resource, 10);
    ```
    */
    pub fn add_resource(&mut self, resource: impl Any) -> Result<(), ECSError> {
        self.resource_storage.insert(resource)
    }

    /**
    Query for a resource and get a mutable reference to it.
    The type of the resource must be added in advance.

    Example:
    ```
    use sara_ecs::World;
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
        self.resource_storage.get_mut::<T>()
    }

    /**
    Retrieves a reference to a resource by its type. If the resource exists, it will be returned as a reference.
    Otherwise, `None` is returned.

    Example:
    ```
    use sara_ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);

    if let Some(resource) = world.get_resource::<u32>() {
        assert_eq!(*resource, 10);
    }
    ```
    */
    pub fn get_resource<T: Any>(&self) -> Option<&T> {
        self.resource_storage.get::<T>()
    }

    /**
    Removes a specific resource from the world by its type. The resource is deleted and can no longer be accessed.

    Example:
    ```
    use sara_ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32);
    world.remove_resource::<u32>();

    assert!(world.get_resource::<u32>().is_none());
    ```
    */
    pub fn remove_resource<T: Any>(&mut self) {
        self.resource_storage.remove::<T>();
    }

    /**
    Checks whether a resource of type `T` exists in the world.

    This function returns `true` if the specified resource type is present in the world; otherwise, it returns `false`.

    Example:
    ```
    use sara_ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32); // Adds a u32 resource to the world

    assert!(world.contains_resource::<u32>()); // Checks if the u32 resource exists
    assert!(!world.contains_resource::<f32>()); // Checks if a f32 resource does not exist
    ```
    */
    pub fn contains_resource<T: Any>(&self) -> bool {
        self.resource_storage.contains::<T>()
    }

    /**
    Replaces an existing resource of type `T` with a new one.

    This function will replace the current resource of the specified type in the world with the provided resource. If no resource of that type exists, the resource will be added.

    Example:
    ```
    use sara_ecs::World;
    let mut world = World::new();

    world.add_resource(10_u32); // Adds a u32 resource to the world
    world.replace_resource(20_u32); // Replaces the u32 resource with a new value

    assert_eq!(world.get_resource::<u32>(), Some(&20)); // Verifies the resource is replaced
    ```
    */
    pub fn replace_resource<T: Any>(&mut self, resource: T) {
        self.resource_storage.replace(resource);
    }

    /**
    Registers a new component type in the world. This component can later be added to entities.
    The type must implement `Any` and have a static lifetime.

    Example:
    ```
    use sara_ecs::World;
    struct Health(pub u32);
    let mut world = World::new();

    world.register_component::<Health>();

    // Health component can now be used for entities
    ```
    */
    pub fn register_component<T: Any + 'static>(&mut self) {
        self.entitiy_storage.register_component::<T>();
    }

    /**
    Creates a new entity. The entity is initially empty and can later be populated with components.
    This function returns a mutable reference to the entity system, allowing you to chain component
    additions to the created entity.

    Example:
    ```
    use sara_ecs::World;
    use sara_ecs::ecs_errors::ECSError;

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
    pub fn create_entity(&mut self) -> &mut entity_storage::EntityStorage {
        self.entitiy_storage.create_entity()
    }

    /**
    Adds a component to an entity by its ID. The component must be registered beforehand.
    This function updates the entity with the provided component data.

    Example:
    ```
    use sara_ecs::World;
    use sara_ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        let entity = world.create_entity().with_component(Health(100))?;

        // Add a new component to the entity with ID 0
        world.add_component_to_entity(0, Speed(15.0))?;

        Ok(())
    }
    ```
    */
    pub fn add_component_to_entity(
        &mut self,
        entity_id: usize,
        component_data: impl Any,
    ) -> Result<(), ECSError> {
        self.entitiy_storage
            .add_component_to_entity(entity_id, component_data)
    }

    /**
    Removes an entity by its ID. The entity and its associated components will be removed from the world.
    If the entity does not exist, an error will be returned.

    Example:
    ```
    use sara_ecs::World;
    use sara_ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        let entity = world.create_entity().with_component(Health(100))?;

        // Remove the entity with ID 0
        world.remove_entity(0)?;

        Ok(())
    }
    ```
    */
    pub fn remove_entity(&mut self, entity_id: usize) -> Result<(), ECSError> {
        self.entitiy_storage.remove_entity(entity_id)
    }

    /**
    Removes a specific component from an entity by its ID. The component type must be registered
    in advance. The function will attempt to remove the component from the entity and return any errors
    if the component is not registered or if there is an issue.

    Example:
    ```
    use sara_ecs::World;
    use sara_ecs::ecs_errors::ECSError;

    struct Health(pub u32);
    struct Speed(pub f32);

    fn example() -> Result<(), ECSError> {
        let mut world = World::new();

        world.register_component::<Health>();
        let entity = world.create_entity().with_component(Health(100))?;

        // Remove the Health component from entity 0
        world.remove_entity_component::<Health>(0)?;

        Ok(())
    }
    ```
    */
    pub fn remove_entity_component<T: Any>(&mut self, entity_id: usize) -> Result<(), ECSError> {
        self.entitiy_storage.remove_entity_component::<T>(entity_id)
    }

    /**
    Query the entities in the world to retrieve components based on the registered component types.
    You can use this to filter and retrieve entities with specific combinations of components.

    Example:
    ```
    use sara_ecs::World;
    use sara_ecs::ecs_errors::ECSError;

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
        Query::new(&self.entitiy_storage)
    }
}
