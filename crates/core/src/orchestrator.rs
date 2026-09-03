//! Runs core systems in dependency order.
//!
//! Each system is followed immediately by its debug-only validation.
//! Transient components used to communicate between systems are removed only
//! after every consumer has run.

use hecs::{Component, Entity, World};

use crate::{
    components::transient::{order::OrderInvalidated, parent::ParentChanged},
    systems::{
        destroy_system::DestroySystem,
        focusing_system::FocusingSystem,
        ordering_system::OrderingSystem,
        parenting_system::{Hierarchy, ParentingSystem},
    },
};

#[cfg(debug_assertions)]
use crate::systems::{focusing_validation, ordering_validation, parenting_validation};

#[derive(Default)]
pub struct Orchestrator {
    destroy: DestroySystem,
    parenting: ParentingSystem,
    ordering: OrderingSystem,
    focusing: FocusingSystem,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, world: &mut World) {
        self.destroy.prepare(world);

        self.parenting.run(world);
        let hierarchy = self.parenting.hierarchy();

        #[cfg(debug_assertions)]
        parenting_validation::validate(world, hierarchy);

        self.ordering.run(world, hierarchy);

        #[cfg(debug_assertions)]
        ordering_validation::validate(world, hierarchy);

        self.focusing.run(world, hierarchy);

        #[cfg(debug_assertions)]
        focusing_validation::validate(world, hierarchy);

        self.destroy.finalize(world);

        cleanup_transients(world);
    }

    pub fn hierarchy(&self) -> &Hierarchy {
        self.parenting.hierarchy()
    }
}

fn cleanup_transients(world: &mut World) {
    remove_component::<ParentChanged>(world);
    remove_component::<OrderInvalidated>(world);
}

fn remove_component<T: Component>(world: &mut World) {
    let entities = world
        .query::<(Entity, &T)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();

    for entity in entities {
        world
            .remove_one::<T>(entity)
            .expect("transient component disappeared during cleanup");
    }
}
