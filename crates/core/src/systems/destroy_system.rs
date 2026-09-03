//! Coordinates safe entity destruction.
//!
//! Destruction happens in two phases. `prepare` marks requested entities with
//! `PendingDestroy`, allowing downstream systems to react while those entities
//! still exist. `finalize` despawns them after structural processing completes.

use hecs::{Entity, World};

use crate::components::{event::destroy::DestroyRequest, transient::destroy::PendingDestroy};

#[derive(Default)]
pub struct DestroySystem;

impl DestroySystem {
    pub fn new() -> Self {
        Self
    }

    pub fn prepare(&mut self, world: &mut World) {
        let requests = world
            .query::<(Entity, &DestroyRequest)>()
            .iter()
            .map(|(event_entity, request)| (event_entity, request.target))
            .collect::<Vec<_>>();

        for (event_entity, target) in requests {
            mark_pending_destroy(world, target);

            world
                .despawn(event_entity)
                .expect("destroy invariant violated: destroy event entity disappeared");
        }
    }

    pub fn finalize(&mut self, world: &mut World) {
        let entities = world
            .query::<Entity>()
            .with::<&PendingDestroy>()
            .iter()
            .collect::<Vec<_>>();

        for entity in entities {
            world
                .despawn(entity)
                .expect("destroy invariant violated: PendingDestroy entity disappeared");
        }
    }
}

fn mark_pending_destroy(world: &mut World, target: Entity) {
    if !world.contains(target) || world.get::<&PendingDestroy>(target).is_ok() {
        return;
    }

    world
        .insert_one(target, PendingDestroy)
        .expect("destroy invariant violated: destroy target disappeared");
}
