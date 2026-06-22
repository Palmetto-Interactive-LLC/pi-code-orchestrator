// Removed: inject_recovery_context relied on delivery::active_md and delivery::inject
// which were superseded by the Temporal-owned delivery model. Recovery context
// injection is now a Temporal activity responsibility.
