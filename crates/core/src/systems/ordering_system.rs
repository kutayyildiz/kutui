use hecs::{Entity, World};

use crate::components::{Order, events::SetOrder};

pub struct OrderingSystem;

impl OrderingSystem {
    pub fn run(world: &mut World) {
        let mut commands = hecs::CommandBuffer::new();

        for (entity, event) in world.query::<(Entity, &SetOrder)>().iter() {
            commands.insert_one(event.target, Order(event.order));

            commands.despawn(entity);
        }

        commands.run_on(world);
    }
}
