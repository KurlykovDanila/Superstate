use bevy_app::{App, Update};
use bevy_ecs::{component::Component, entity::Entity, query::With, system::Query};
use superstate::{superstate_plugin, SuperstateInfo};

#[derive(Component, Default)]
#[require(SuperstateInfo<Movement>)]
struct Movement;

#[derive(Component)]
#[require(Movement)]
struct Walking;

#[derive(Component)]
#[require(Movement)]
struct Running;

#[derive(Component)]
#[require(Movement)]
struct Flying;

fn main() {
    App::new()
        .add_plugins(superstate_plugin::<Movement, (Walking, Running, Flying)>)
        .add_systems(Update, (on_super_state, concrete_state))
        .run();
}

fn on_super_state(q: Query<Entity, With<Movement>>) {
    // the system will be called in any of the states
    if !q.is_empty() {
        println!("Contain some Movement")
    }
}

fn concrete_state(q: Query<Entity, With<Walking>>) {
    // the system will be called on state Walking
    if !q.is_empty() {
        println!("State is Walking and no other states!");
    }
}
