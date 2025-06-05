//! # Example
//!
//! ```
//! use superstate::{superstate_plugin, SuperstateInfo};
//! use bevy_app::App;
//! use bevy_ecs::component::Component;
//!
//! #[derive(Default, Component)]
//! #[require(SuperstateInfo<SuperA>)]
//! struct SuperA;
//!
//! #[derive(Component)]
//! #[require(SuperA)]
//! struct A1;
//!
//! #[derive(Component)]
//! #[require(SuperA)]
//! struct A2;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(superstate_plugin::<SuperA, (A1, A2)>)
//!         .run();
//! }
//! ```
//!
//! To register different super states, call the plugin with different types.
//!
//! ```
//! use superstate::{superstate_plugin, SuperstateInfo};
//! use bevy_app::App;
//! use bevy_ecs::component::Component;
//!
//! #[derive(Default, Component)]
//! #[require(SuperstateInfo<SuperA>)]
//! struct SuperA;
//!
//! #[derive(Component)]
//! #[require(SuperA)]
//! struct A1;
//!
//! #[derive(Component)]
//! #[require(SuperA)]
//! struct A2;
//!
//! #[derive(Default, Component)]
//! #[require(SuperstateInfo<SuperB>)]
//! struct SuperB;
//!
//! #[derive(Component)]
//! #[require(SuperB)]
//! struct B1;
//!
//! #[derive(Component)]
//! #[require(SuperB)]
//! struct B2;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(superstate_plugin::<SuperA, (A1, A2)>)
//!         .add_plugins(superstate_plugin::<SuperB, (B1, B2)>)
//!         .run();
//! }
//! ```

use std::marker::PhantomData;

use bevy_app::App;
use bevy_ecs::{
    bundle::Bundle,
    component::{Component, ComponentId},
    error::BevyError,
    world::World,
};
use hooks::HookBusyError;

pub mod hooks {
    use std::{error::Error, fmt::Display};

    use bevy_ecs::{
        bundle::Bundle,
        component::{Component, ComponentId, HookContext},
        world::DeferredWorld,
    };

    use crate::SuperstateInfo;

    /// When registering components as states or as super states,
    /// a case may occur where the component hooks are already registered.
    /// For example, if you use the same components for relationships.
    /// Currently, this limitation cannot be bypassed, since a component can only have one hook.
    #[derive(Debug, Clone)]
    pub struct HookBusyError(pub ComponentId);

    impl Display for HookBusyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Hook on this component({:?}) is busy.", self.0)
        }
    }

    impl Error for HookBusyError {}

    /// Hook that called when adding any state component from `States`.
    /// Removes the rest of the state components because the state should be unique.
    /// If you add multiple states to an entity at once, only the last new one will remain.
    pub fn on_add_hook_state<Super: Component, States: Bundle>(
        mut world: DeferredWorld,
        ctx: HookContext,
    ) {
        let mut ids = Vec::new();
        States::get_component_ids(world.components(), &mut |id| ids.push(id.unwrap()));
        let (mut entities, mut cmd) = world.entities_and_commands();
        let mut entity = entities.get_mut(ctx.entity).unwrap();
        let mut info = entity.get_mut::<SuperstateInfo<Super>>().unwrap();
        if info.state_ids.len() == 0 {
            info.state_ids = ids.into();
        }
        info.states_on_entity.push(ctx.component_id);
        for id in info.states_on_entity.iter() {
            if *id != ctx.component_id {
                cmd.entity(ctx.entity).remove_by_id(*id);
            }
        }
    }

    /// Hook that called when removing any state component from `States`.
    /// Remove `Super` componet if no others states.
    pub fn on_remove_hook_state<Super: Component, States: Bundle>(
        mut world: DeferredWorld,
        ctx: HookContext,
    ) {
        let (mut entities, mut cmd) = world.entities_and_commands();
        let mut entity = entities.get_mut(ctx.entity).unwrap();
        let mut info = entity.get_mut::<SuperstateInfo<Super>>().unwrap();
        info.remove_by_id(ctx.component_id);
        if info.states_on_entity.is_empty() {
            cmd.entity(ctx.entity).remove::<Super>();
        }
    }

    /// Hook that called when adding `Super` component.
    /// If you try inset `Super` component when no any states component on entity, `Super` no will be added.
    pub fn on_add_superstate<Super: Component, States: Bundle>(
        mut world: DeferredWorld,
        ctx: HookContext,
    ) {
        let (entities, mut cmd) = world.entities_and_commands();
        let entity = entities.get(ctx.entity).unwrap();
        let info = entity.get::<SuperstateInfo<Super>>().unwrap();
        if info.states_on_entity.is_empty() {
            cmd.entity(ctx.entity).remove::<Super>();
        }
    }

    /// Hook that called when removing `Super` component. Remove all `States`.
    pub fn on_remove_superstate<Super: Component, States: Bundle>(
        mut world: DeferredWorld,
        ctx: HookContext,
    ) {
        let (mut entities, mut cmd) = world.entities_and_commands();
        let mut entity = entities.get_mut(ctx.entity).unwrap();
        let mut info = entity.get_mut::<SuperstateInfo<Super>>().unwrap();
        cmd.entity(ctx.entity).remove::<States>();
        info.states_on_entity.clear();
    }
}

/// Just call [`register_hooks`].
///
/// -`Super` - superstate component type.
///
/// -`States` - bundle with all concrete states component types.
///
pub fn superstate_plugin<Super: Component, States: Bundle>(app: &mut App) {
    register_hooks::<Super, States>(app.world_mut()).unwrap();
}

/// Called when building a plugin to register component hooks.
/// Use this function if you are not using the [`App`] and only work with the [`World`].
///
/// Registers on_add: [`hooks::on_add_hook_state`], and
/// on_remove: [`hooks::on_remove_hook_state`] component hooks
/// for every state component.
///
/// Registers on_add: [`hooks::on_add_superstate`], and
/// on_remove: [`hooks::on_remove_superstate`] component hooks
/// for superstate component.
///
/// More details about each hook can be found in the [`hooks`] module.
pub fn register_hooks<Super: Component, States: Bundle>(
    world: &mut World,
) -> Result<(), BevyError> {
    let super_id = world.register_component::<Super>();
    let states_ids = world
        .register_bundle::<States>()
        .iter_explicit_components()
        .collect::<Vec<_>>();
    for id in states_ids {
        world
            .register_component_hooks_by_id(id)
            .ok_or(HookBusyError(id))?
            .on_add(hooks::on_add_hook_state::<Super, States>)
            .on_remove(hooks::on_remove_hook_state::<Super, States>);
    }
    world
        .register_component_hooks_by_id(super_id)
        .ok_or(HookBusyError(super_id))?
        .on_add(hooks::on_add_superstate::<Super, States>)
        .on_remove(hooks::on_remove_superstate::<Super, States>);
    Ok(())
}

/// A component for storing auxiliary information to ensure
/// that only one state exists at a time. Used in component hooks.
/// Type `S` is a superstate component type.
///
/// The component initialization requires dynamic memory allocations,
/// and is never deleted once created, even if the entity
/// has no state components left. You can safely delete this component
/// if you verify that the entity does not have a superstate.
/// It is recommended to delete this component if your entity
/// will no longer accept previously registered states.
#[derive(Component, Default, Debug, Clone)]
pub struct SuperstateInfo<S: Component> {
    state_ids: Box<[ComponentId]>,
    // Vector of states which on entity on one momemet.
    // Can be more than 1, when user spawn entity with
    // several different states.
    states_on_entity: Vec<ComponentId>,
    _p: PhantomData<S>,
}

impl<S: Component> SuperstateInfo<S> {
    fn remove_by_id(&mut self, id: ComponentId) {
        // Find item`s index with equal ComponentId.
        if let Some((index, _)) = self
            .states_on_entity
            .iter()
            .enumerate()
            .find(|(_, _id)| **_id == id)
        {
            // Never panic, because index never out of bounds.
            self.states_on_entity.swap_remove(index);
        }
    }
}
