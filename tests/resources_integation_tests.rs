use sara_ecs::{ecs_errors::ECSError, World};

struct FpsResource(pub u32);

fn get_test_world() -> Result<World, ECSError> {
    let mut world = World::new();

    world.add_resource(FpsResource(60))?;
    Ok(world)
}

#[test]
fn create_and_get_resource_immutably() {
    let world = get_test_world().unwrap();

    dbg!(&world);

    let fps = world.get_resource::<FpsResource>().unwrap();

    assert_eq!(fps.0, 60);
}

#[test]
fn get_resources_mutably() {
    let mut world = get_test_world().unwrap();

    {
        let fps = world.get_resource_mut::<FpsResource>().unwrap();
        fps.0 += 1;
    }

    let fps = world.get_resource_mut::<FpsResource>().unwrap();

    assert_eq!(fps.0, 61);
}

#[test]
fn remove_resources() {
    let mut world = get_test_world().unwrap();
    world.remove_resource::<FpsResource>();

    let removed_resource = world.get_resource::<FpsResource>();

    assert!(removed_resource.is_none());
}

#[test]
fn contains_resource() {
    let world = get_test_world().unwrap();

    assert!(world.contains_resource::<FpsResource>());
    assert!(!world.contains_resource::<i32>());
}

#[test]
fn replace_resource() {
    let mut world = get_test_world().unwrap();

    world.replace_resource(FpsResource(120));

    let fps = world.get_resource::<FpsResource>().unwrap();
    assert_eq!(fps.0, 120);
}
