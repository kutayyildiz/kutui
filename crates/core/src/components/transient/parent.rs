use hecs::Entity;

pub struct ParentChanged {
    pub previous: Option<Entity>,
}
