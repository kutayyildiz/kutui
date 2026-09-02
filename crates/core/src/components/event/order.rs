use hecs::Entity;

#[derive(Clone, Copy)]
pub enum RequestOrder {
    Set { target: Entity, order: usize },
    Increment { target: Entity },
    Decrement { target: Entity },
}
