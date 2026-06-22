// Removed: ACTIVE_LEASES, GENERATIONS, generate_lease, validate_lease,
// and increment_generation were in-memory lease tracking superseded by the
// Temporal-owned delivery model. The durability authority is Temporal; local
// in-memory counters no longer serve a purpose in the production code path.
