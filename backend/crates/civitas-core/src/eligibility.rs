//! Pure eligibility-policy evaluation.
//!
//! "Can this user vote?" is a boolean function of (a) the user's verification
//! state and (b) the policy in effect for the proposal. Both inputs are
//! plain data; the function has no I/O.
//!
//! v1 ships two policies. The intent is that future verification methods —
//! phone SMS, `WebAuthn`, government e-ID — slot in as additional policy
//! variants without disturbing call sites.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use civitas_types::UserId;

/// Verification timestamps for a user. `None` means "not verified by that
/// method."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserVerificationStatus {
    pub user_id: UserId,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub phone_verified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl UserVerificationStatus {
    /// Convenience: a user with verified email and nothing else.
    #[must_use]
    pub fn email_only(user_id: UserId, when: DateTime<Utc>) -> Self {
        Self {
            user_id,
            email_verified_at: Some(when),
            phone_verified_at: None,
            deleted_at: None,
        }
    }
}

/// What the proposal (or deployment) requires for a user to be eligible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityPolicy {
    /// Email verification only. The v1 default.
    EmailVerified,
    /// Both email and phone verified.
    EmailAndPhoneVerified,
}

/// Pure check: does `status` satisfy `policy`?
///
/// Returns `false` for soft-deleted users regardless of policy.
#[must_use]
pub fn is_eligible(status: &UserVerificationStatus, policy: &EligibilityPolicy) -> bool {
    if status.deleted_at.is_some() {
        return false;
    }
    match policy {
        EligibilityPolicy::EmailVerified => status.email_verified_at.is_some(),
        EligibilityPolicy::EmailAndPhoneVerified => {
            status.email_verified_at.is_some() && status.phone_verified_at.is_some()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn t() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn unverified_user_fails_email_policy() {
        let s = UserVerificationStatus {
            user_id: UserId::new(),
            email_verified_at: None,
            phone_verified_at: None,
            deleted_at: None,
        };
        assert!(!is_eligible(&s, &EligibilityPolicy::EmailVerified));
    }

    #[test]
    fn email_verified_satisfies_email_policy() {
        let s = UserVerificationStatus::email_only(UserId::new(), t());
        assert!(is_eligible(&s, &EligibilityPolicy::EmailVerified));
    }

    #[test]
    fn email_only_fails_email_and_phone_policy() {
        let s = UserVerificationStatus::email_only(UserId::new(), t());
        assert!(!is_eligible(&s, &EligibilityPolicy::EmailAndPhoneVerified));
    }

    #[test]
    fn both_satisfies_combined_policy() {
        let s = UserVerificationStatus {
            user_id: UserId::new(),
            email_verified_at: Some(t()),
            phone_verified_at: Some(t()),
            deleted_at: None,
        };
        assert!(is_eligible(&s, &EligibilityPolicy::EmailAndPhoneVerified));
    }

    #[test]
    fn deleted_user_never_eligible() {
        let s = UserVerificationStatus {
            user_id: UserId::new(),
            email_verified_at: Some(t()),
            phone_verified_at: Some(t()),
            deleted_at: Some(t()),
        };
        assert_eq!(is_eligible(&s, &EligibilityPolicy::EmailVerified), false);
        assert_eq!(
            is_eligible(&s, &EligibilityPolicy::EmailAndPhoneVerified),
            false
        );
    }
}
