use hecs::Entity;

#[derive(Clone, Copy)]
pub enum OrderRequest {
    Set { target: Entity, order: usize },
    Increment { target: Entity },
    Decrement { target: Entity },
}
