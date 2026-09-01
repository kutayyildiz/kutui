use hecs::Entity;

// Event
pub struct DestroyEntity {
    pub target: Entity,
}

// Component
pub struct PendingDestroy;
