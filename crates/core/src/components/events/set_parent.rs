use hecs::Entity;

pub struct SetParent {
    pub target: Entity,
    pub parent: Entity,
}
