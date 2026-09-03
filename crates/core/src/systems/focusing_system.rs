//! Maintains focus within the entity hierarchy.
//!
//! Every parent has exactly one focused direct child.
//!
//! `ParentingSystem` runs before `OrderingSystem`, which runs before this
//! system. The hierarchy and sibling order are therefore authoritative.
//!
//! Only `FocusingSystem` may add or remove `Focused`, and only this system
//! consumes `FocusRequest` events.

use hecs::{Entity, World};

use crate::{
    components::{
        event::focus::FocusRequest,
        state::{focus::Focused, order::Order, parent::Parent},
        transient::{order::OrderInvalidated, parent::ParentChanged},
    },
    systems::parenting_system::Hierarchy,
};

#[derive(Default)]
pub struct FocusingSystem;

impl FocusingSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, world: &mut World, hierarchy: &Hierarchy) {
        repair_changed_parents(world, hierarchy);
        process_focus_requests(world, hierarchy);
    }
}

fn repair_changed_parents(world: &mut World, hierarchy: &Hierarchy) {
    //
    // Pass 1:
    //
    // A focused child changed parent.
    //
    // Its Focused marker belongs to its previous parent relationship, so:
    //
    // 1. Snapshot every such entity.
    // 2. Remove Focused from all of them.
    // 3. Repair their previous parents.
    //
    // Removing all old focus first ensures a child newly focused during repair
    // cannot have that new focus removed later in this pass.
    //

    let changed_focused = world
        .query::<(Entity, &ParentChanged, &OrderInvalidated, &Focused)>()
        .iter()
        .map(|(entity, parent_changed, order_changed, _)| {
            (entity, parent_changed.previous, order_changed.previous)
        })
        .collect::<Vec<_>>();

    for &(entity, _, _) in &changed_focused {
        remove_focus(world, entity);
    }

    for (_, previous_parent, previous_order) in changed_focused {
        if let Some(previous_parent) = previous_parent
            && let Some(children) = hierarchy.get(&previous_parent)
        {
            let focus_order = previous_order.min(children.len() - 1);

            for &child in children {
                if order_of(world, child) == focus_order {
                    add_focus(world, child);
                    break;
                }
            }
        }
    }

    //
    // Pass 2:
    //
    // Every changed child that is currently unfocused is considered within
    // its new parent.
    //
    // If the new parent already has a focused child, nothing is needed.
    // Otherwise this child becomes focused.
    //

    let changed_unfocused = world
        .query::<(Entity, &ParentChanged, &Parent)>()
        .without::<&Focused>()
        .iter()
        .map(|(entity, _, parent)| (entity, parent.0))
        .collect::<Vec<_>>();

    for (entity, parent) in changed_unfocused {
        if let Some(children) = hierarchy.get(&parent)
            && !children.iter().any(|child| has_focus(world, *child))
        {
            add_focus(world, entity);
        }
    }
}

fn process_focus_requests(world: &mut World, hierarchy: &Hierarchy) {
    let requests = world
        .query::<(Entity, &FocusRequest)>()
        .iter()
        .map(|(entity, request)| (entity, request.target))
        .collect::<Vec<_>>();

    for (event_entity, target) in requests {
        set_focus(world, hierarchy, target);

        world
            .despawn(event_entity)
            .expect("focusing invariant violated: focus request disappeared");
    }
}

fn set_focus(world: &mut World, hierarchy: &Hierarchy, target: Entity) {
    if let Some(parent) = parent_of(world, target)
        && !has_focus(world, target)
        && let Some(children) = hierarchy.get(&parent)
        && children.contains(&target)
    {
        for &child in children {
            if child != target && has_focus(world, child) {
                remove_focus(world, child);
            }
        }
        add_focus(world, target);
    }
}

fn parent_of(world: &World, entity: Entity) -> Option<Entity> {
    world.get::<&Parent>(entity).ok().map(|parent| parent.0)
}

fn order_of(world: &World, entity: Entity) -> usize {
    world
        .get::<&Order>(entity)
        .expect("ordering invariant violated: hierarchy child has no Order")
        .0
}

fn has_focus(world: &World, entity: Entity) -> bool {
    world.get::<&Focused>(entity).is_ok()
}

fn add_focus(world: &mut World, entity: Entity) {
    world
        .insert_one(entity, Focused)
        .expect("focusing invariant violated: focus target disappeared");
}

fn remove_focus(world: &mut World, entity: Entity) {
    world
        .remove_one::<Focused>(entity)
        .expect("focusing invariant violated: focused entity disappeared");
}
