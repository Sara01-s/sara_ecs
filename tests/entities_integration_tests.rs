use sara_ecs::ecs_errors::ECSError;
use sara_ecs::World;

struct Position(pub f32, pub f32);
struct Scale(pub f32, pub f32);

#[test]
fn create_entity() -> Result<(), ECSError> {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Scale>();

    world
        .create_entity()
        .with_component(Position(0.0, 0.0))?
        .with_component(Scale(0.0, 0.0))?;

    Ok(())
}

#[test]
fn query_for_entities() -> Result<(), ECSError> {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Scale>();

    world
        .create_entity()
        .with_component(Position(2.0, -3.0))?
        .with_component(Scale(1.0, 2.0))?;

    world
        .create_entity()
        .with_component(Position(5.0, -10.0))?
        .with_component(Scale(10.0, 30.0))?;

    world.create_entity().with_component(Position(99.0, -9.0))?;
    world.create_entity().with_component(Scale(-45.0, 45.0))?;

    let query = world
        .query()
        .with_component_filter::<Position>()?
        .with_component_filter::<Scale>()?
        .run();

    let positions = &query.components[0];
    let scales = &query.components[1];

    assert_eq!(positions.len(), scales.len());
    assert_eq!(positions.len(), 2);

    let borrowed_first_position = positions[0].borrow();
    let first_position = borrowed_first_position.downcast_ref::<Position>().unwrap();
    assert_eq!(first_position.0, 2.0);
    assert_eq!(first_position.1, -3.0);

    let borrowed_first_scale = scales[0].borrow();
    let first_scale = borrowed_first_scale.downcast_ref::<Scale>().unwrap();
    assert_eq!(first_scale.0, 1.0);
    assert_eq!(first_scale.1, 2.0);

    let borrowed_second_position = positions[1].borrow();
    let second_position = borrowed_second_position.downcast_ref::<Position>().unwrap();
    assert_eq!(second_position.0, 5.0);

    let mut borrowed_second_scale = scales[1].borrow_mut();
    let second_scale = borrowed_second_scale.downcast_mut::<Scale>().unwrap();
    second_scale.0 *= 10.0;
    assert_eq!(second_scale.0, 100.0);
    Ok(())
}

#[test]
fn remove_component_from_entity() -> Result<(), ECSError> {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Scale>();

    world
        .create_entity()
        .with_component(Position(0.0, 0.0))?
        .with_component(Scale(1.0, 1.0))?;

    world
        .create_entity()
        .with_component(Position(5.0, 5.0))?
        .with_component(Scale(2.0, 2.0))?;

    world.remove_entity_component::<Position>(0)?;

    let query = world
        .query()
        .with_component_filter::<Position>()?
        .with_component_filter::<Scale>()?
        .run();

    assert_eq!(query.entity_ids.len(), 1);
    assert_eq!(query.entity_ids[0], 1);
    Ok(())
}

#[test]
fn add_component_to_entity() -> Result<(), ECSError> {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Scale>();

    world.create_entity().with_component(Position(1.0, 1.0))?;

    world.add_component_to_entity(0, Scale(20.0, 50.0))?;

    let query = world
        .query()
        .with_component_filter::<Position>()?
        .with_component_filter::<Scale>()?
        .run();

    assert_eq!(query.entity_ids.len(), 1);
    Ok(())
}

#[test]
fn deleting_an_entity() -> Result<(), ECSError> {
    let mut world = World::new();

    world.register_component::<Position>();
    world.register_component::<Scale>();

    world.create_entity().with_component(Position(1.0, 1.0))?;
    world.create_entity().with_component(Position(2.0, 3.0))?;

    world.remove_entity(0)?;

    let query = world.query().with_component_filter::<Position>()?.run();

    assert_eq!(query.entity_ids.len(), 1);

    let borrowed_position = query.components[0][0].borrow();
    let position = borrowed_position.downcast_ref::<Position>().unwrap();

    assert_eq!(position.0, 2.0);
    assert_eq!(position.1, 3.0);

    world.create_entity().with_component(Position(30.0, 35.0))?;

    let query = world.query().with_component_filter::<Position>()?.run();
    let borrowed_position = query.components[0][0].borrow();
    let position = borrowed_position.downcast_ref::<Position>().unwrap();

    assert_eq!(position.0, 30.0);
    assert_eq!(position.1, 35.0);
    Ok(())
}
