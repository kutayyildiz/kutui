use hecs::Entity;

pub struct ClearParent {
    pub target: Entity,
}

pub struct ClearChildren {
    pub target: Entity,
}

pub struct SetParent {
    pub target: Entity,
    pub parent: Entity,
}
