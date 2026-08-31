use hecs::Entity;

pub struct SetOrder {
    pub target: Entity,
    pub order: usize,
}
