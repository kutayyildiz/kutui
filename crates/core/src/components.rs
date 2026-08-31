use hecs::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent {
    pub entity: Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Order {
    pub value: usize,
}
