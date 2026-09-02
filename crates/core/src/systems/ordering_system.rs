//! Maintains dense ordering within sibling groups.
//!
//! `ParentingSystem` runs before this system and provides the authoritative
//! hierarchy. Parent changes are exposed through the transient `ParentChanged`
//! component.
//!
//! Only this system may add, modify, or remove `Order`.

use std::collections::HashSet;

use hecs::{Entity, World};

use crate::{
    components::{
        event::order::OrderRequest, state::order::Order, state::parent::Parent,
        transient::parent::ParentChanged,
    },
    systems::parenting_system::Hierarchy,
};

#[derive(Default)]
pub struct OrderingSystem;

impl OrderingSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, world: &mut World, hierarchy: &Hierarchy) {
        reconcile_structure(world, hierarchy);
        process_order_requests(world, hierarchy);

        #[cfg(debug_assertions)]
        validate_ordering(world, hierarchy);
    }
}

fn reconcile_structure(world: &mut World, hierarchy: &Hierarchy) {
    let mut affected_parents = HashSet::new();
    let mut detached = Vec::new();

    for (entity, changed, parent, order) in world
        .query::<(Entity, &ParentChanged, Option<&Parent>, Option<&Order>)>()
        .iter()
    {
        if let Some(previous) = changed.previous {
            affected_parents.insert(previous);
        }

        if let Some(parent) = parent {
            affected_parents.insert(parent.0);
        } else if order.is_some() {
            detached.push(entity);
        }
    }

    for (_, parent) in world
        .query::<(Entity, &Parent)>()
        .without::<&Order>()
        .iter()
    {
        affected_parents.insert(parent.0);
    }

    for entity in detached {
        world
            .remove_one::<Order>(entity)
            .expect("ordering invariant violated: detached entity lost its Order");
    }

    for parent in affected_parents {
        normalize_children(world, hierarchy, parent);
    }
}

fn normalize_children(world: &mut World, hierarchy: &Hierarchy, parent: Entity) {
    let Some(children) = hierarchy.get(&parent) else {
        return;
    };

    let mut existing = Vec::new();
    let mut appended = Vec::new();

    for &child in children {
        let parent_changed = world.get::<&ParentChanged>(child).is_ok();
        let order = world.get::<&Order>(child).ok().map(|order| order.0);

        if parent_changed || order.is_none() {
            appended.push(child);
        } else {
            existing.push((child, order.unwrap()));
        }
    }

    existing.sort_unstable_by_key(|(_, order)| *order);

    #[cfg(debug_assertions)]
    for pair in existing.windows(2) {
        assert_ne!(
            pair[0].1, pair[1].1,
            "ordering invariant violated: duplicate Order within sibling group"
        );
    }

    let ordered = existing
        .into_iter()
        .map(|(entity, _)| entity)
        .chain(appended);

    for (order, entity) in ordered.enumerate() {
        write_order(world, entity, order);
    }
}

fn write_order(world: &mut World, entity: Entity, order: usize) {
    if let Ok(mut current) = world.get::<&mut Order>(entity) {
        current.0 = order;
        return;
    }

    world
        .insert_one(entity, Order(order))
        .expect("ordering invariant violated: ordered entity disappeared");
}

fn process_order_requests(world: &mut World, hierarchy: &Hierarchy) {
    let requests = world
        .query::<(Entity, &OrderRequest)>()
        .iter()
        .map(|(entity, request)| (entity, *request))
        .collect::<Vec<_>>();

    for (event_entity, request) in requests {
        match request {
            OrderRequest::Set { target, order } => {
                set_order(world, hierarchy, target, order);
            }

            OrderRequest::Increment { target } => {
                increment_order(world, hierarchy, target);
            }

            OrderRequest::Decrement { target } => {
                decrement_order(world, hierarchy, target);
            }
        }

        world
            .despawn(event_entity)
            .expect("ordering invariant violated: ordering event entity disappeared");
    }
}

fn move_to_index(
    world: &World,
    siblings: &HashSet<Entity>,
    target: Entity,
    current: usize,
    destination: usize,
) {
    if current == destination {
        return;
    }

    for &sibling in siblings {
        if sibling == target {
            continue;
        }

        let mut order = world
            .get::<&mut Order>(sibling)
            .expect("ordering invariant violated: sibling has no Order");

        if destination < current {
            if order.0 >= destination && order.0 < current {
                order.0 += 1;
            }
        } else if order.0 > current && order.0 <= destination {
            order.0 -= 1;
        }
    }

    world
        .get::<&mut Order>(target)
        .expect("ordering invariant violated: target lost its Order")
        .0 = destination;
}

fn set_order(world: &World, hierarchy: &Hierarchy, target: Entity, requested: usize) {
    let Some((siblings, current)) = resolve_target(world, hierarchy, target) else {
        return;
    };

    let destination = requested.min(siblings.len() - 1);

    move_to_index(world, siblings, target, current, destination);
}

fn increment_order(world: &World, hierarchy: &Hierarchy, target: Entity) {
    let Some((siblings, current)) = resolve_target(world, hierarchy, target) else {
        return;
    };

    let destination = if current == siblings.len() - 1 {
        0
    } else {
        current + 1
    };

    move_to_index(world, siblings, target, current, destination);
}

fn decrement_order(world: &World, hierarchy: &Hierarchy, target: Entity) {
    let Some((siblings, current)) = resolve_target(world, hierarchy, target) else {
        return;
    };

    let destination = if current == 0 {
        siblings.len() - 1
    } else {
        current - 1
    };

    move_to_index(world, siblings, target, current, destination);
}

fn resolve_target<'a>(
    world: &World,
    hierarchy: &'a Hierarchy,
    target: Entity,
) -> Option<(&'a HashSet<Entity>, usize)> {
    let parent = world.get::<&Parent>(target).ok()?.0;

    let siblings = hierarchy
        .get(&parent)
        .expect("ordering invariant violated: Parent has no hierarchy entry");

    assert!(
        siblings.contains(&target),
        "ordering invariant violated: hierarchy does not contain ordering target"
    );

    let current = world
        .get::<&Order>(target)
        .expect("ordering invariant violated: ordering target has no Order")
        .0;

    Some((siblings, current))
}

#[cfg(debug_assertions)]
fn validate_ordering(world: &World, hierarchy: &Hierarchy) {
    for (entity, _) in world.query::<(Entity, &Order)>().iter() {
        assert!(
            world.get::<&Parent>(entity).is_ok(),
            "ordering invariant violated: entity with Order has no Parent"
        );
    }

    for (entity, _) in world.query::<(Entity, &Parent)>().iter() {
        assert!(
            world.get::<&Order>(entity).is_ok(),
            "ordering invariant violated: parented entity has no Order"
        );
    }

    for children in hierarchy.values() {
        let mut orders = children
            .iter()
            .map(|&child| {
                world
                    .get::<&Order>(child)
                    .expect("ordering invariant violated: hierarchy child has no Order")
                    .0
            })
            .collect::<Vec<_>>();

        orders.sort_unstable();

        for (expected, actual) in orders.into_iter().enumerate() {
            assert_eq!(
                actual, expected,
                "ordering invariant violated: sibling group is not dense"
            );
        }
    }
}
