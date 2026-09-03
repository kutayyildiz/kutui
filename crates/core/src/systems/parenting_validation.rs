//! Debug validation for `ParentingSystem`.

use std::collections::HashMap;

use hecs::{Entity, World};

use crate::{
    components::{state::parent::Parent, transient::parent::ParentChanged},
    systems::parenting_system::Hierarchy,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Visit {
    Visiting,
    Done,
}

pub(crate) fn validate(world: &World, hierarchy: &Hierarchy) {
    validate_parent_components(world, hierarchy);
    validate_hierarchy(world, hierarchy);
    validate_parent_changes(world);
    validate_no_cycles(world);
}

fn validate_parent_components(world: &World, hierarchy: &Hierarchy) {
    for (child, parent) in world.query::<(Entity, &Parent)>().iter() {
        assert!(
            world.contains(parent.0),
            "parenting invariant violated: Parent references a missing entity"
        );

        let children = hierarchy
            .get(&parent.0)
            .expect("parenting invariant violated: Parent has no hierarchy entry");

        assert!(
            children.contains(&child),
            "parenting invariant violated: Parent is missing from hierarchy"
        );
    }
}

fn validate_hierarchy(world: &World, hierarchy: &Hierarchy) {
    for (&parent, children) in hierarchy {
        assert!(
            world.contains(parent),
            "parenting invariant violated: hierarchy parent does not exist"
        );

        assert!(
            !children.is_empty(),
            "parenting invariant violated: hierarchy contains an empty child group"
        );

        for &child in children {
            let actual_parent = world
                .get::<&Parent>(child)
                .expect("parenting invariant violated: hierarchy child has no Parent");

            assert_eq!(
                actual_parent.0, parent,
                "parenting invariant violated: hierarchy disagrees with Parent"
            );
        }
    }
}
fn validate_parent_changes(world: &World) {
    for (_, changed, parent) in world
        .query::<(Entity, &ParentChanged, Option<&Parent>)>()
        .iter()
    {
        let current = parent.map(|parent| parent.0);

        assert_ne!(
            changed.previous, current,
            "parenting invariant violated: ParentChanged does not represent a change"
        );
    }
}

fn validate_no_cycles(world: &World) {
    let mut visited = HashMap::new();

    for entity in world.query::<Entity>().with::<&Parent>().iter() {
        if visited.get(&entity) == Some(&Visit::Done) {
            continue;
        }

        let mut path = Vec::new();
        let mut current = entity;

        loop {
            match visited.get(&current) {
                Some(Visit::Visiting) => {
                    panic!(
                        "parenting invariant violated: hierarchy contains a cycle at {current:?}"
                    );
                }
                Some(Visit::Done) => break,
                None => {}
            }

            visited.insert(current, Visit::Visiting);
            path.push(current);

            let Some(parent) = world.get::<&Parent>(current).ok().map(|parent| parent.0) else {
                break;
            };

            current = parent;
        }

        for entity in path {
            visited.insert(entity, Visit::Done);
        }
    }
}
