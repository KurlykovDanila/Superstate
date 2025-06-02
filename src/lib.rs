use std::marker::PhantomData;

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

    #[derive(Debug, Clone)]
    pub struct HookBusyError(pub ComponentId);

    impl Display for HookBusyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Hook on this component({:?}) is busy.", self.0)
        }
    }

    impl Error for HookBusyError {}

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
        let index = self
            .states_on_entity
            .iter()
            .enumerate()
            .find(|(_, _id)| **_id == id)
            .unwrap()
            .0;
        self.states_on_entity.swap_remove(index);
    }
}
