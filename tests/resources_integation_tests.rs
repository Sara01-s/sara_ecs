use sara_ecs::World;

fn get_test_world() -> World {
    let mut world = World::new();
    world.add_resource(FpsResource(60));
    world
}

struct FpsResource(pub u32);

#[test]
fn create_and_get_resource_immutably() {
    let world = get_test_world();

    dbg!(&world);

    let fps = world.get_resource::<FpsResource>().unwrap();
    assert_eq!(fps.0, 60);
}

#[test]
fn get_resources_mutably() {
    let mut world = get_test_world();

    {
        let fps = world.get_resource_mut::<FpsResource>().unwrap();
        fps.0 += 1;
    }

    let fps = world.get_resource_mut::<FpsResource>().unwrap();
    assert_eq!(fps.0, 61);
}

#[test]
fn remove_resources() {
    let mut world = get_test_world();

    world.remove_resource::<FpsResource>();

    let removed_resource = world.get_resource::<FpsResource>();
    assert!(removed_resource.is_none());
}
