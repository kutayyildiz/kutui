//! Debug validation for `FocusingSystem`.

use hecs::{Entity, World};

use crate::{
    components::state::{focus::Focused, parent::Parent},
    systems::parenting_system::Hierarchy,
};

pub(crate) fn validate(world: &World, hierarchy: &Hierarchy) {
    validate_one_focused_child_per_parent(world, hierarchy);
    validate_focused_entities_are_parented(world);
}

fn validate_one_focused_child_per_parent(world: &World, hierarchy: &Hierarchy) {
    for (&parent, children) in hierarchy {
        let focused = children
            .iter()
            .filter(|&&child| world.get::<&Focused>(child).is_ok())
            .count();

        assert_eq!(
            focused, 1,
            "focusing invariant violated: parent {parent:?} does not have exactly one focused child"
        );
    }
}

fn validate_focused_entities_are_parented(world: &World) {
    for entity in world.query::<Entity>().with::<&Focused>().iter() {
        assert!(
            world.get::<&Parent>(entity).is_ok(),
            "focusing invariant violated: focused entity has no Parent"
        );
    }
}
