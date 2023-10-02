use std::sync::{Arc, Mutex};

use bevy_ecs::{prelude::*, system::BoxedSystem};

use crate::EntityEvent;

#[derive(Default, Debug, Clone)]
pub enum CallbackSystem {
    #[default]
    Empty,
    New(Arc<Mutex<BoxedSystem>>),
    Initialized(Arc<Mutex<BoxedSystem>>),
}

impl CallbackSystem {
    pub(crate) fn is_initialized(&self) -> bool {
        matches!(self, CallbackSystem::Initialized(_))
    }

    pub(crate) fn run(&mut self, world: &mut World) {
        if !self.is_initialized() {
            let mut temp = CallbackSystem::Empty;
            std::mem::swap(self, &mut temp);
            if let CallbackSystem::New(system) = temp {
                system.lock().unwrap().initialize(world);
                *self = CallbackSystem::Initialized(system);
            }
        }
        if let CallbackSystem::Initialized(system) = self {
            system.lock().unwrap().run((), world);
            system.lock().unwrap().apply_deferred(world);
        }
    }
}

/// A [`SystemParam`](bevy::ecs::system::SystemParam) used to get immutable access the the
/// [`ListenerInput`] for this callback.
///
/// Use this in callback systems to access event data for the event that triggered the callback.
pub type Listener<'w, E> = Res<'w, ListenerInput<E>>;

/// A [`SystemParam`](bevy::ecs::system::SystemParam) used to get mutable access the the
/// [`ListenerInput`] for this callback.
///
/// Use this in callback systems to access event data for the event that triggered the callback.
pub type ListenerMut<'w, E> = ResMut<'w, ListenerInput<E>>;

/// Data from an event that triggered an [`On<Event>`](crate::event_listener::On) listener, and is
/// currently bubbling through the entity hierarchy.
///
/// This is accessed as a bevy resource in the callback system. This resource is only available to
/// callback systems.
///
/// ```
/// # use bevy_eventlistener_core::{callbacks::ListenerMut, event_listener::EntityEvent};
/// # use bevy_ecs::prelude::*;
/// # #[derive(Clone, Event)]
/// # struct MyEvent {
/// #     target: Entity,
/// #     foo: usize,
/// # }
/// # impl EntityEvent for MyEvent {
/// #     fn target(&self) -> Entity {
/// #         self.target
/// #     }
/// # }
/// fn my_callback(mut event: ListenerMut<MyEvent>) {
///     event.foo += 1; // Mutate the event that is being bubbled
///     event.target(); // The entity that was originally targeted
///     event.listener(); // The entity that was listening for this event
///     event.stop_propagation(); // Stop the event from bubbling further
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Resource)]
pub struct ListenerInput<E: EntityEvent> {
    /// The entity that was listening for this event.
    pub(crate) listener: Entity,
    /// Event-specific information.
    pub(crate) event_data: E,
    pub(crate) propagate: bool,
}

impl<E: EntityEvent> ListenerInput<E> {
    /// The entity that was listening for this event. Call `target()` to get the entity that this
    /// event originally targeted before it started bubbling through the hierarchy. Note that the
    /// target and listener can be the same entity.
    pub fn listener(&self) -> Entity {
        self.listener
    }

    /// When called, the event will stop bubbling up the hierarchy to its parent.
    pub fn stop_propagation(&mut self) {
        self.propagate = false;
    }
}

impl<E: EntityEvent> std::ops::Deref for ListenerInput<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event_data
    }
}

impl<E: EntityEvent> std::ops::DerefMut for ListenerInput<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event_data
    }
}
