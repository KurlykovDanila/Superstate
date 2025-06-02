#[cfg(test)]
mod invariant_test {
    use bevy_ecs::bundle::Bundle;
    use bevy_ecs::entity::Entity;
    use bevy_ecs::query::{Or, With};
    use bevy_ecs::system::Query;
    use bevy_ecs::world::unsafe_world_cell;
    use bevy_ecs::{component::Component, world::World};
    use superstate::register_hooks;
    use superstate::SuperstateInfo;

    #[derive(Default, Component)]
    #[require(SuperstateInfo<Movement>)]
    struct Movement;

    #[derive(Component)]
    #[require(Movement)]
    struct Walking(u32);

    #[derive(Component)]
    #[require(Movement)]
    struct Running(u32);

    #[derive(Component)]
    #[require(Movement)]
    struct Flying(u32);

    #[test]
    fn main() {
        let mut world = World::new();
        register_hooks::<Movement, (Walking, Running, Flying)>(&mut world).unwrap();
        let no_states = world.register_system(no_states_and_superstate_system);
        let walking = world.register_system(has_check_state::<Movement, Walking>);
        let running = world.register_system(has_check_state::<Movement, Running>);
        let flying = world.register_system(has_check_state::<Movement, Flying>);
        let no_walking = world.register_system(has_not_check_state::<Movement, Walking>);
        let no_running = world.register_system(has_not_check_state::<Movement, Running>);
        let no_flying = world.register_system(has_not_check_state::<Movement, Flying>);
        let e = world.spawn_empty().id();
        world.run_system(no_states).unwrap();
        world.commands().entity(e).insert(Walking(10));
        world.run_system(walking).unwrap();
        world.run_system(no_running).unwrap();
        world.run_system(no_flying).unwrap();
        world.commands().entity(e).insert(Running(11));
        world.run_system(running).unwrap();
        world.run_system(no_walking).unwrap();
        world.run_system(no_flying).unwrap();
        world.commands().entity(e).remove::<Running>();
        world.run_system(no_states).unwrap();
        world.commands().entity(e).insert(Flying(12));
        world.run_system(flying).unwrap();
        world.commands().entity(e).remove::<Movement>();
        world.run_system(no_states).unwrap();
    }

    fn no_states_and_superstate_system(
        q: Query<Entity, Or<(With<Movement>, With<Running>, With<Flying>, With<Walking>)>>,
    ) {
        assert!(q.is_empty());
    }

    fn has_check_state<Super: Component, State: Component>(
        q: Query<Entity, (With<State>, With<Super>)>,
    ) {
        assert!(!q.is_empty());
    }

    fn has_not_check_state<Super: Component, State: Component>(
        q: Query<Entity, (With<State>, With<Super>)>,
    ) {
        assert!(q.is_empty());
    }
}
