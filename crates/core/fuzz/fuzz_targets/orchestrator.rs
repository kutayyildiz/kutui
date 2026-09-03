#![no_main]

use arbitrary::Arbitrary;
use hecs::{Entity, World};
use libfuzzer_sys::fuzz_target;

use kutui_core::{
    Orchestrator,
    components::event::{
        destroy::DestroyRequest, focus::FocusRequest, order::OrderRequest, parent::ParentRequest,
    },
};

const MAX_ENTITIES: usize = 16;
const MAX_OPERATIONS: usize = 256;

#[derive(Debug, Arbitrary)]
enum Operation {
    Spawn { slot: u8 },
    Destroy { target: u8 },

    SetParent { target: u8, parent: u8 },
    ClearParent { target: u8 },
    ClearChildren { target: u8 },

    SetOrder { target: u8, order: u8 },
    IncrementOrder { target: u8 },
    DecrementOrder { target: u8 },

    Focus { target: u8 },

    Run,
}

fuzz_target!(|operations: Vec<Operation>| {
    let mut world = World::new();
    let mut orchestrator = Orchestrator::new();
    let mut entities = [None; MAX_ENTITIES];

    for operation in operations.into_iter().take(MAX_OPERATIONS) {
        apply_operation(&mut world, &mut orchestrator, &mut entities, operation);
    }

    // Process any requests remaining at the end of the input.
    orchestrator.run(&mut world);
});

fn apply_operation(
    world: &mut World,
    orchestrator: &mut Orchestrator,
    entities: &mut [Option<Entity>; MAX_ENTITIES],
    operation: Operation,
) {
    match operation {
        Operation::Spawn { slot } => {
            spawn(world, entities, slot);
        }

        Operation::Destroy { target } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((DestroyRequest { target },));
        }

        Operation::SetParent { target, parent } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            let Some(parent) = resolve(world, entities, parent) else {
                return;
            };

            world.spawn((ParentRequest::Set { target, parent },));
        }

        Operation::ClearParent { target } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((ParentRequest::Clear { target },));
        }

        Operation::ClearChildren { target } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((ParentRequest::ClearChildren { target },));
        }

        Operation::SetOrder { target, order } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((OrderRequest::Set {
                target,
                order: usize::from(order),
            },));
        }

        Operation::IncrementOrder { target } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((OrderRequest::Increment { target },));
        }

        Operation::DecrementOrder { target } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((OrderRequest::Decrement { target },));
        }

        Operation::Focus { target } => {
            let Some(target) = resolve(world, entities, target) else {
                return;
            };

            world.spawn((FocusRequest { target },));
        }

        Operation::Run => {
            orchestrator.run(world);
        }
    }
}

fn spawn(world: &mut World, entities: &mut [Option<Entity>; MAX_ENTITIES], slot: u8) {
    let slot = slot_index(slot);

    if entities[slot].is_some_and(|entity| world.contains(entity)) {
        return;
    }

    entities[slot] = Some(world.spawn(()));
}

fn resolve(world: &World, entities: &[Option<Entity>; MAX_ENTITIES], slot: u8) -> Option<Entity> {
    let entity = entities[slot_index(slot)]?;

    world.contains(entity).then_some(entity)
}

fn slot_index(slot: u8) -> usize {
    usize::from(slot) % MAX_ENTITIES
}
