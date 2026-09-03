/// Preserves an entity's order from its previous sibling group.
///
/// Added by `OrderingSystem` when an entity leaves a parent and its existing
/// `Order` therefore becomes invalid for its new hierarchy position.
///
/// `previous` is the entity's order within its previous parent's child group.
/// It is preserved so downstream systems can reason about the position the
/// entity occupied before the parent change.
///
/// This is not emitted for ordinary ordering changes such as `OrderRequest`.
/// It specifically represents order invalidation caused by a parent change.
///
/// Removed during orchestrator transient cleanup.
pub struct OrderInvalidated {
    pub previous: usize,
}
