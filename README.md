# superstate

`Superstate` is a simple and easy to use bevy ecs plugin of typed states and super states of entity components.

## Installation
Add superstate as a dependency to Cargo.toml:
```toml
[dependencies]
superstate = { git = "https://github.com/KurlykovDanila/Superstate.git" }
```
Now the crate is not published on crates.io

## Problem
When we have a component responsible for a state, we have two ways, the first one is that one component is responsible for one state, which means we have to express it using enums:
```rust
#[derive(Component)]
enum Movement{
  Walking,
  Running,
  Fyling,
}
```
and later use this component in a similar way (forced to check its value every time):
```rust
fn running_system(q: Single<(Entity, &Movement)>) {
    let (e, move_state) = q.into_inner();
    // forced to do a check in each such system, 
    // and if there are dozens of states? it looks very dirty
    match move_state {
        Movement::Running => {///running logic}
        _ => {}
    }
} 
```
Another way is to declare each state as a separate type: 
```rust
#[derive(Component)]
struct Walking;

#[derive(Component)]
struct Running;

#[derive(Component)]
struct Flying;
```
it looks cleaner (independent systems that operate without knowing how many states there may be) at first glance:
```rust
fn walking_system(q: Single<(Entity, With<Walking>)>) {
    // no state checks, the request did everything for us,
    // we can concentrate only on the logic of a specific state.
} 

```
however, as soon as we want to change one state to another, we have to remember all the others too...
```rust
// the problem is in the query, we have to enter all possible states
fn change_state_system(mut cmd: Commands, q: Single<(Entity, Or<(With<Walking>, With<Running>, With<Flying>)>)>) {
    // since we don't know the current state, we have to delete all of
    // them and then insert the one we wanted to change the state to.
    cmd.entity(q.entity()).remove::<(Walking, Running, Flying)>();
    cmd.entity(q.entity()).insert::<Walking>();
} 
```

## Solution
The solution is component hooks and required components. Make superstate requried of states.
```rust
use superstate::{SuperstateInfo};

#[derive(Component, Default)]
#[require(SuperstateInfo<Movement>)]
struct Movement; // superstate

#[derive(Component)]
#[require(Movement)]
struct Walking; // state

#[derive(Component)]
#[require(Movement)]
struct Running; // state

#[derive(Component)]
#[require(Movement)]
struct Flying; // state
```
add plugin
```rust
use superstate::{superstate_plugin};

App::new()
    .add_plugins(superstate_plugin::<Movement, (Walking, Running, Flying)>)
    ...
    .run();
```
use systems for superstate
```rust
fn on_super_state(q: Query<Entity, With<Movement>>) {
    // the system will be called in any of the states
    if !q.is_empty() {
        println!("Contain some Movement")
    }
}
```
use systems for concrete state type
```rust
fn concrete_state(q: Query<Entity, With<Walking>>) {
    // the system will be called on state Walking
    if !q.is_empty() {
        println!("State is Walking and no other states!");
    }
}
```
## Features
- Automatically remove previous state when adding new one
- Automatically remove a concrete state when a super state component is deleted.
- Avoids the error of adding a super state without a specific state. (just not adding to the entity)
- The super state and the specific state always exist together. If one is not there, the other is not there either.
