use bevy::reflect::Reflect;
use bevy::render::view::RenderLayers;
use bevy::{prelude::*, utils::HashMap};

use crate::body::AnimBodyBundle;
use crate::lazy::{impl_get_copy, impl_with_on};
use crate::traits::AnimStateMachine;

#[derive(Clone, Debug, Reflect)]
pub(crate) struct AnimResetStateInfo<StateMachine: AnimStateMachine> {
    pub(crate) state: StateMachine,
    pub(crate) ix: u32,
    pub(crate) time: f32,
}

/// The main animation controller
#[derive(Debug, Clone, Reflect)]
pub struct AnimMan<StateMachine: AnimStateMachine> {
    /// Current state of the animation
    pub(crate) state: StateMachine,
    /// Used internally for changing state
    pub(crate) reset_state: Option<AnimResetStateInfo<StateMachine>>,
    /// Used internally for changing flip
    pub(crate) reset_flip: bool,
    /// Flips x-axis of the animation
    pub(crate) flip_x: bool,
    /// Flips y-axis of the animation
    pub(crate) flip_y: bool,
    /// Repeat this animation x
    pub(crate) rep_x: u32,
    /// Repeat this animation y
    pub(crate) rep_y: u32,
    /// Should the `AnimStateChange` event be triggered?
    pub(crate) observe_state_changes: bool,
    /// Should the `AnimIxChange` event be triggered?
    pub(crate) observe_ix_changes: bool,
    /// Use this render layer instead of that specified in the animation
    pub(crate) render_layers: Option<RenderLayers>,
    /// Is this animation guaranteed to only use one state? If so, only spawn one child
    /// TODO: Deprecate this is stupid (I think)
    pub(crate) singular: bool,
    /// INTERNAL: The entities of the spawned body children
    pub(crate) tagged_children: HashMap<StateMachine, Entity>,
}
impl<StateMachine: AnimStateMachine> Default for AnimMan<StateMachine> {
    fn default() -> Self {
        Self {
            state: default(),
            reset_state: Some(AnimResetStateInfo {
                state: default(),
                ix: 0,
                time: 0.0,
            }),
            reset_flip: false,
            flip_x: false,
            flip_y: false,
            rep_x: 1,
            rep_y: 1,
            observe_state_changes: false,
            observe_ix_changes: false,
            render_layers: None,
            singular: false,
            tagged_children: default(),
        }
    }
}
impl<StateMachine: AnimStateMachine> AnimMan<StateMachine> {
    pub fn new(state: StateMachine) -> Self {
        Self {
            state,
            reset_state: Some(AnimResetStateInfo {
                state,
                ix: 0,
                time: 0.0,
            }),
            ..default()
        }
    }
    pub fn with_state(mut self, val: StateMachine) -> Self {
        self.state = val;
        self.reset_state.as_mut().unwrap().state = val;
        self
    }
    pub fn with_initial_ix(mut self, ix: u32) -> Self {
        if let Some(reset_state) = self.reset_state.as_mut() {
            reset_state.ix = ix;
        }
        self
    }
    pub fn with_flip_x(mut self, val: bool) -> Self {
        self.flip_x = val;
        self.reset_flip = true;
        self
    }
    pub fn with_flip_y(mut self, val: bool) -> Self {
        self.flip_y = val;
        self.reset_flip = true;
        self
    }
    pub fn with_rep_x(mut self, val: u32) -> Self {
        self.rep_x = val;
        self
    }
    pub fn with_rep_y(mut self, val: u32) -> Self {
        self.rep_y = val;
        self
    }
    impl_with_on!(observe_state_changes);
    impl_with_on!(observe_ix_changes);
    pub fn with_render_layers(mut self, rl: RenderLayers) -> Self {
        self.render_layers = Some(rl);
        self
    }
    impl_with_on!(singular);
}
impl<StateMachine: AnimStateMachine> AnimMan<StateMachine> {
    pub fn get_state(&self) -> StateMachine {
        self.reset_state
            .as_ref()
            .map(|reset| reset.state)
            .unwrap_or(self.state)
    }
    impl_get_copy!(flip_x, bool);
    impl_get_copy!(flip_y, bool);
    impl_get_copy!(rep_x, u32);
    impl_get_copy!(rep_y, u32);

    /// If the given state is equal to the current state, nothing happens.
    /// Otherwise, the state is changed to the given state, and the animation is reset to the first frame.
    pub fn set_state(&mut self, state: StateMachine) {
        if self.state != state {
            self.reset_state = Some(AnimResetStateInfo {
                state,
                ix: 0,
                time: 0.0,
            });
        }
    }
    /// The given state is set, and the animation is reset to the first frame.
    pub fn reset_state(&mut self, state: StateMachine) {
        self.reset_state = Some(AnimResetStateInfo {
            state,
            ix: 0,
            time: 0.0,
        });
    }
    /// Set the flipx value of the animation
    pub fn set_flip_x(&mut self, flip_x: bool) {
        if flip_x != self.flip_x {
            self.flip_x = flip_x;
            self.reset_flip = true;
        }
    }
    /// Set the flipy value of the animation
    pub fn set_flip_y(&mut self, flip_y: bool) {
        if flip_y != self.flip_y {
            self.flip_y = flip_y;
            self.reset_flip = true;
        }
    }
}

impl<StateMachine: AnimStateMachine> Component for AnimMan<StateMachine> {
    const STORAGE_TYPE: bevy::ecs::component::StorageType =
        bevy::ecs::component::StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, eid, _| {
            let myself = world
                .get::<Self>(eid)
                .expect("AnimState: on_add hook should have myself");
            let flip_x = myself.flip_x;
            let flip_y = myself.flip_y;
            let rep_x = myself.rep_x;
            let rep_y = myself.rep_y;
            let my_state = myself.state;
            let render_layers_override = myself.render_layers.clone();
            let singular = myself.singular;
            let mut tagged_children = HashMap::default();
            for state in StateMachine::all().into_iter() {
                if singular && state != my_state {
                    continue;
                }
                let bund = AnimBodyBundle::new(
                    state,
                    flip_x,
                    flip_y,
                    rep_x,
                    rep_y,
                    state == my_state,
                    render_layers_override.clone(),
                    &mut world,
                );
                let mut commands = world.commands();
                let mut ent_comms = commands.spawn(bund);
                ent_comms.set_parent(eid);
                let child_eid = ent_comms.id();
                tagged_children.insert(state, child_eid);
            }
            let mut mut_myself = world
                .get_mut::<Self>(eid)
                .expect("AnimState: on_add hook should have myself2");
            mut_myself.tagged_children = tagged_children;
        });
    }
}
