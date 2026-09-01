use hecs::{Entity, World};

use crate::components::{
    Parent,
    events::{
        parent::{ClearChildren, ClearParent, SetParent},
    },
};


impl ParentingSystem {
    pub fn run(world: &mut World) {
        let mut commands = hecs::CommandBuffer::new();

        // Process SetParent events before ClearParent events so that, when both target
        // the same entity in one tick, ClearParent deterministically wins.

        // Set Parents
        for (entity, event) in world.query::<(Entity, &SetParent)>().iter() {
            if world.contains(event.parent) {
                commands.insert_one(event.target, Parent(event.parent));
            }

            commands.despawn(entity);
        }

        // Clear Parents
        for (entity, event) in world.query::<(Entity, &ClearParent)>().iter() {
            commands.remove_one::<Parent>(event.target);
            commands.despawn(entity);
        }

        commands.run_on(world);
    }
}
