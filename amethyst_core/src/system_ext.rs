//! Extension system utilities.
//!
//! This modules contains an extension trait for the System trait which adds useful transformation
//! functions.

use shred::{RunningTime, SystemData};
use specs::prelude::{Read, System};

/// Extension functionality associated systems.
pub trait SystemExt {
    /// Make a system pausable by tying it to a specific value of a resource.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate amethyst;
    /// use amethyst::{
    ///     ecs::{System, Write},
    ///     shred::DispatcherBuilder,
    ///     prelude::*,
    /// };
    ///
    /// #[derive(PartialEq)]
    /// enum CurrentState {
    ///     Disabled,
    ///     Enabled,
    /// }
    ///
    /// impl Default for CurrentState {
    ///     fn default() -> Self {
    ///         CurrentState::Disabled
    ///     }
    /// }
    ///
    /// struct AddNumber(u32);
    ///
    /// impl<'s> System<'s> for AddNumber {
    ///     type SystemData = Write<'s, u32>;
    ///
    ///     fn run(&mut self, mut number: Self::SystemData) {
    ///         *number += self.0;
    ///     }
    /// }
    ///
    /// let mut world = World::new();
    ///
    /// let mut dispatcher = DispatcherBuilder::default()
    ///     .with(AddNumber(1), "set_number", &[])
    ///     .with(AddNumber(2).pausable(CurrentState::Enabled), "set_number_2", &[])
    ///     .build();
    ///
    /// dispatcher.setup(&mut world.res);
    ///
    /// // we only expect the u32 resource to be modified _if_ the system is enabled,
    /// // the system should only be enabled on CurrentState::Enabled.
    ///
    /// *world.write_resource() = 0u32;
    /// dispatcher.dispatch(&mut world.res);
    /// assert_eq!(1, *world.read_resource::<u32>());
    ///
    /// *world.write_resource() = 0u32;
    /// *world.write_resource() = CurrentState::Enabled;
    /// dispatcher.dispatch(&mut world.res);
    /// assert_eq!(1 + 2, *world.read_resource::<u32>());
    /// ```
    fn pausable<V: 'static>(self, value: V) -> Pausable<Self, V>
    where
        Self: Sized,
        V: Send + Sync + Default + PartialEq;
}

impl<'s, S> SystemExt for S
where
    S: System<'s>,
{
    fn pausable<V: 'static>(self, value: V) -> Pausable<Self, V>
    where
        Self: Sized,
        V: Send + Sync + Default + PartialEq,
    {
        Pausable {
            system: self,
            value,
        }
    }
}

/// A system that is enabled when `U` has a specific value.
pub struct Pausable<S, V> {
    system: S,
    value: V,
}

impl<'s, S, V: 'static> System<'s> for Pausable<S, V>
where
    S::SystemData: SystemData<'s>,
    S: System<'s>,
    V: Send + Sync + Default + PartialEq,
{
    type SystemData = (Read<'s, V>, S::SystemData);

    fn run(&mut self, data: Self::SystemData) {
        if self.value != *data.0 {
            return;
        }

        self.system.run(data.1);
    }

    fn running_time(&self) -> RunningTime {
        self.system.running_time()
    }
}
