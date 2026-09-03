use hecs::Entity;

#[derive(Clone, Copy)]
pub enum ParentRequest {
    Set { target: Entity, parent: Entity },
    Clear { target: Entity },
    ClearChildren { target: Entity },
}
