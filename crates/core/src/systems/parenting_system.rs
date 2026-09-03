//! Maintains `Parent` components and the derived parent-to-children hierarchy.
//!
//! Other systems request hierarchy changes through parenting events. Only this
//! system may modify hierarchy state or consume parenting event entities.
//!
//! `PendingDestroy` is treated as a hard constraint before normal parenting
//! priority is resolved. The resulting hierarchy is kept acyclic. Hierarchy
//! depth limits and parent/child kind validation are intentionally out of scope.

use std::collections::{HashMap, HashSet};

use hecs::{Entity, World};

use crate::components::{
    event::parent::ParentRequest,
    state::parent::Parent,
    transient::{destroy::PendingDestroy, parent::ParentChanged},
};

pub type Hierarchy = HashMap<Entity, HashSet<Entity>>;

const PARENTING_PRIORITY: ParentingPriority = ParentingPriority::Clear;

enum ParentingPriority {
    // Supported for future configurations; current policy uses Clear.
    #[allow(dead_code)]
    Set,
    Clear,
}

#[derive(Default)]
struct CollectedParentingEvents {
    set_parent: Vec<(Entity, Entity)>,
    clear_parent: HashSet<Entity>,
    clear_children: HashSet<Entity>,
    entities: Vec<Entity>,
}

struct ParentingEvents {
    set_parent: HashMap<Entity, Entity>,
    clear_parent: HashSet<Entity>,
    clear_children: HashSet<Entity>,
    entities: Vec<Entity>,
}

#[derive(Default)]
pub struct ParentingSystem {
    children_by_parent: Option<Hierarchy>,
}

impl ParentingSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, world: &mut World) {
        self.ensure_hierarchy(world);

        let pending_destroy = query_pending_destroy(world);
        let collected = query_parenting_events(world);
        let mut events = normalize_parenting_events(world, collected, &pending_destroy);

        let mut destroy_clears = build_destroy_clears(world, self.hierarchy(), &pending_destroy);

        resolve_destroy_conflicts(&events.set_parent, &mut destroy_clears);

        lower_clear_children(self.hierarchy(), &mut events);
        merge_destroy_clears(&mut events, destroy_clears);
        resolve_parenting_priority(&mut events, PARENTING_PRIORITY);
        remove_noops(world, &mut events);

        remove_cyclic_sets(world, &mut events, &pending_destroy);

        apply_parenting_events(world, self.hierarchy_mut(), &events);
        cleanup_hierarchy(self.hierarchy_mut());
        consume_parenting_events(world, events.entities);
    }

    fn ensure_hierarchy(&mut self, world: &World) {
        if self.children_by_parent.is_some() {
            return;
        }

        let mut hierarchy = Hierarchy::new();

        for (child, parent) in world.query::<(Entity, &Parent)>().iter() {
            hierarchy.entry(parent.0).or_default().insert(child);
        }

        self.children_by_parent = Some(hierarchy);
    }

    pub fn hierarchy(&self) -> &Hierarchy {
        self.children_by_parent
            .as_ref()
            .expect("parenting hierarchy must be initialized")
    }

    fn hierarchy_mut(&mut self) -> &mut Hierarchy {
        self.children_by_parent
            .as_mut()
            .expect("parenting hierarchy must be initialized")
    }
}

fn query_pending_destroy(world: &World) -> HashSet<Entity> {
    world
        .query::<Entity>()
        .with::<&PendingDestroy>()
        .iter()
        .map(|entity| entity)
        .collect()
}

fn query_parenting_events(world: &World) -> CollectedParentingEvents {
    let mut collected = CollectedParentingEvents::default();

    for (event_entity, request) in world.query::<(Entity, &ParentRequest)>().iter() {
        collected.entities.push(event_entity);

        match *request {
            ParentRequest::Set { target, parent } => {
                collected.set_parent.push((target, parent));
            }
            ParentRequest::Clear { target } => {
                collected.clear_parent.insert(target);
            }
            ParentRequest::ClearChildren { target } => {
                collected.clear_children.insert(target);
            }
        }
    }

    collected
}

fn normalize_parenting_events(
    world: &World,
    collected: CollectedParentingEvents,
    pending_destroy: &HashSet<Entity>,
) -> ParentingEvents {
    let mut set_parent = HashMap::new();

    for (child, parent) in collected.set_parent {
        if world.contains(child)
            && world.contains(parent)
            && !pending_destroy.contains(&child)
            && !pending_destroy.contains(&parent)
        {
            // HECS query order is arbitrary by design. The first valid request
            // encountered for a child survives; later requests are discarded.
            set_parent.entry(child).or_insert(parent);
        }
    }

    ParentingEvents {
        set_parent,
        clear_parent: collected.clear_parent,
        clear_children: collected.clear_children,
        entities: collected.entities,
    }
}

fn build_destroy_clears(
    world: &World,
    hierarchy: &Hierarchy,
    pending_destroy: &HashSet<Entity>,
) -> HashSet<Entity> {
    let mut clears = HashSet::new();

    for &entity in pending_destroy {
        // Dev: this can also be done with an additional single hecs query: Parent && PendingDestroy. Decide which is cleaner/efficient.
        if world.get::<&Parent>(entity).is_ok() {
            clears.insert(entity);
        }
        if let Some(children) = hierarchy.get(&entity) {
            clears.extend(children.iter().copied());
        }
    }

    clears
}

fn resolve_destroy_conflicts(
    set_parent: &HashMap<Entity, Entity>,
    destroy_clears: &mut HashSet<Entity>,
) {
    // All surviving sets have non-destroyed endpoints. Therefore a surviving
    // set may safely reparent a child that was detached by destruction.
    destroy_clears.retain(|child| !set_parent.contains_key(child));
}

fn lower_clear_children(hierarchy: &Hierarchy, events: &mut ParentingEvents) {
    for parent in &events.clear_children {
        if let Some(children) = hierarchy.get(parent) {
            events.clear_parent.extend(children.iter().copied());
        }
    }

    for (&child, &parent) in &events.set_parent {
        if events.clear_children.contains(&parent) {
            events.clear_parent.insert(child);
        }
    }
    // clear_children should not be used again
}

fn merge_destroy_clears(events: &mut ParentingEvents, destroy_clears: HashSet<Entity>) {
    // destroy_clears can now safely merge into clear_parent events
    events.clear_parent.extend(destroy_clears);
}

fn resolve_parenting_priority(events: &mut ParentingEvents, priority: ParentingPriority) {
    match priority {
        ParentingPriority::Clear => {
            let clear_parent = &events.clear_parent;
            events
                .set_parent
                .retain(|child, _| !clear_parent.contains(child));
        }
        ParentingPriority::Set => {
            let set_parent = &events.set_parent;
            events
                .clear_parent
                .retain(|child| !set_parent.contains_key(child));
        }
    }
}

fn remove_noops(world: &World, events: &mut ParentingEvents) {
    events
        .clear_parent
        .retain(|child| world.get::<&Parent>(*child).is_ok());

    events.set_parent.retain(
        |child, requested_parent| match world.get::<&Parent>(*child) {
            Ok(current_parent) => current_parent.0 != *requested_parent,
            Err(_) => true,
        },
    );
}

fn apply_parenting_events(world: &mut World, hierarchy: &mut Hierarchy, events: &ParentingEvents) {
    for &child in &events.clear_parent {
        let previous_parent = world
            .get::<&Parent>(child)
            .expect("parenting invariant violated: clear target lost its Parent")
            .0;

        world
            .remove_one::<Parent>(child)
            .expect("parenting invariant violated: failed to remove Parent");

        world
            .insert_one(
                child,
                ParentChanged {
                    previous: Some(previous_parent),
                },
            )
            .expect("parenting invariant violated: failed to record ParentChanged");

        remove_hierarchy_edge(hierarchy, previous_parent, child);
    }

    for (&child, &new_parent) in &events.set_parent {
        let previous_parent = world.get::<&Parent>(child).ok().map(|parent| parent.0);

        if let Some(previous_parent) = previous_parent {
            remove_hierarchy_edge(hierarchy, previous_parent, child);
        }

        world
            .insert_one(child, Parent(new_parent))
            .expect("parenting invariant violated: SetParent target disappeared");

        world
            .insert_one(
                child,
                ParentChanged {
                    previous: previous_parent,
                },
            )
            .expect("parenting invariant violated: failed to record ParentChanged");

        let inserted = hierarchy.entry(new_parent).or_default().insert(child);

        assert!(
            inserted,
            "hierarchy invariant violated: new parent already contained child"
        );
    }
}

fn remove_hierarchy_edge(hierarchy: &mut Hierarchy, parent: Entity, child: Entity) {
    let children = hierarchy
        .get_mut(&parent)
        .expect("hierarchy invariant violated: Parent has no hierarchy entry");

    assert!(
        children.remove(&child),
        "hierarchy invariant violated: hierarchy entry does not contain child"
    );
}

fn cleanup_hierarchy(hierarchy: &mut Hierarchy) {
    hierarchy.retain(|_, children| !children.is_empty());
}

fn consume_parenting_events(world: &mut World, entities: Vec<Entity>) {
    for entity in entities {
        world
            .despawn(entity)
            .expect("parenting invariant violated: parenting event entity disappeared");
    }
}

fn remove_cyclic_sets(
    world: &World,
    events: &mut ParentingEvents,
    pending_destroy: &HashSet<Entity>,
) {
    let mut planned = world
        .query::<(Entity, &Parent)>()
        .iter()
        .map(|(child, parent)| (child, parent.0))
        .collect::<HashMap<_, _>>();

    // Apply planned clears first.
    for child in &events.clear_parent {
        planned.remove(child);
    }

    // Assume every surviving SetParent succeeds.
    for (&child, &parent) in &events.set_parent {
        planned.insert(child, parent);
    }

    // Reject one cyclic SetParent at a time until the planned graph is acyclic.
    while let Some(child) = find_cyclic_set(&planned, &events.set_parent) {
        events.set_parent.remove(&child);

        if let Ok(parent) = world.get::<&Parent>(child) {
            if pending_destroy.contains(&parent.0) {
                planned.remove(&child);
                events.clear_parent.insert(child);
            } else {
                planned.insert(child, parent.0);
            }
        } else {
            planned.remove(&child);
        }
    }
}

fn find_cyclic_set(
    planned: &HashMap<Entity, Entity>,
    set_parent: &HashMap<Entity, Entity>,
) -> Option<Entity> {
    set_parent
        .keys()
        .copied()
        .find(|&child| is_cyclic(planned, child))
}

fn is_cyclic(planned: &HashMap<Entity, Entity>, child: Entity) -> bool {
    let mut current = child;

    for _ in 0..=planned.len() {
        let Some(&parent) = planned.get(&current) else {
            return false;
        };

        if parent == child {
            return true;
        }

        current = parent;
    }

    true
}
