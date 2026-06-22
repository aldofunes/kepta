/// Free tier limits — enforced at API level.
/// Billing can be disabled entirely for self-hosted instances.
pub struct FreeTierLimits;

impl FreeTierLimits {
    pub const HOUSEHOLDS: i32 = 1;
    pub const MEMBERS_PER_HOUSEHOLD: i32 = 2;
    pub const DOCUMENTS_PER_HOUSEHOLD: i32 = 25;
    pub const STORAGE_MB_PER_HOUSEHOLD: i32 = 500;
}

#[derive(Debug, Clone)]
pub struct HouseholdUsage {
    pub household_count: i64,
    pub member_count: i64,
    pub document_count: i64,
    pub storage_bytes: i64,
}

#[derive(Debug, Clone)]
pub struct PlanLimits {
    pub household_limit: Option<i32>,
    pub member_limit: Option<i32>,
    pub document_limit: Option<i32>,
    pub storage_limit_mb: Option<i32>,
}

impl PlanLimits {
    pub fn unlimited() -> Self {
        Self {
            household_limit: None,
            member_limit: None,
            document_limit: None,
            storage_limit_mb: None,
        }
    }

    pub fn free() -> Self {
        Self {
            household_limit: Some(FreeTierLimits::HOUSEHOLDS),
            member_limit: Some(FreeTierLimits::MEMBERS_PER_HOUSEHOLD),
            document_limit: Some(FreeTierLimits::DOCUMENTS_PER_HOUSEHOLD),
            storage_limit_mb: Some(FreeTierLimits::STORAGE_MB_PER_HOUSEHOLD),
        }
    }

    pub fn can_add_household(&self, current: i64) -> bool {
        self.household_limit.map_or(true, |limit| current < limit as i64)
    }

    pub fn can_add_member(&self, current: i64) -> bool {
        self.member_limit.map_or(true, |limit| current < limit as i64)
    }

    pub fn can_add_document(&self, current: i64) -> bool {
        self.document_limit.map_or(true, |limit| current < limit as i64)
    }

    pub fn can_add_storage(&self, current_bytes: i64, new_bytes: i64) -> bool {
        self.storage_limit_mb.map_or(true, |limit| {
            current_bytes + new_bytes < (limit as i64) * 1024 * 1024
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Free tier

    #[test]
    fn free_allows_first_household() {
        assert!(PlanLimits::free().can_add_household(0));
    }

    #[test]
    fn free_blocks_second_household() {
        assert!(!PlanLimits::free().can_add_household(1));
    }

    #[test]
    fn free_allows_up_to_member_limit() {
        let limits = PlanLimits::free();
        assert!(limits.can_add_member(0));
        assert!(limits.can_add_member(1));
        assert!(!limits.can_add_member(2));
    }

    #[test]
    fn free_blocks_at_document_limit() {
        let limits = PlanLimits::free();
        assert!(limits.can_add_document(24));
        assert!(!limits.can_add_document(25));
    }

    #[test]
    fn free_blocks_when_storage_would_exceed_limit() {
        let limits = PlanLimits::free();
        let limit_bytes = (FreeTierLimits::STORAGE_MB_PER_HOUSEHOLD as i64) * 1024 * 1024;
        assert!(limits.can_add_storage(0, limit_bytes - 1));
        assert!(!limits.can_add_storage(0, limit_bytes));
        assert!(!limits.can_add_storage(limit_bytes - 1, 1));
    }

    // Unlimited (pro)

    #[test]
    fn unlimited_never_blocks() {
        let limits = PlanLimits::unlimited();
        assert!(limits.can_add_household(1_000_000));
        assert!(limits.can_add_member(1_000_000));
        assert!(limits.can_add_document(1_000_000));
        assert!(limits.can_add_storage(i64::MAX / 2, i64::MAX / 2));
    }
}
