//! Debug validation for `ParentingSystem`.

use hecs::{Entity, World};

use crate::{
    components::{state::parent::Parent, transient::parent::ParentChanged},
    systems::parenting_system::Hierarchy,
};

pub(crate) fn validate(world: &World, hierarchy: &Hierarchy) {
    validate_parent_components(world, hierarchy);
    validate_hierarchy(world, hierarchy);
    validate_parent_changes(world);
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
