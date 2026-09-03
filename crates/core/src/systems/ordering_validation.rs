//! Debug validation for `OrderingSystem`.

use hecs::{Entity, World};

use crate::{
    components::state::{order::Order, parent::Parent},
    systems::parenting_system::Hierarchy,
};

pub(crate) fn validate(world: &World, hierarchy: &Hierarchy) {
    validate_order_membership(world);
    validate_dense_orders(world, hierarchy);
}

fn validate_order_membership(world: &World) {
    for (entity, _) in world.query::<(Entity, &Parent)>().iter() {
        assert!(
            world.get::<&Order>(entity).is_ok(),
            "ordering invariant violated: parented entity has no Order"
        );
    }

    for (entity, _) in world.query::<(Entity, &Order)>().iter() {
        assert!(
            world.get::<&Parent>(entity).is_ok(),
            "ordering invariant violated: entity with Order has no Parent"
        );
    }
}

fn validate_dense_orders(world: &World, hierarchy: &Hierarchy) {
    for children in hierarchy.values() {
        let mut orders = children
            .iter()
            .map(|&child| {
                world
                    .get::<&Order>(child)
                    .expect("ordering invariant violated: ordered child has no Order")
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
