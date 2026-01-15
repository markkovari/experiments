use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheck {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::serde_helpers::serialize",
        deserialize_with = "crate::serde_helpers::deserialize"
    )]
    pub id: Option<String>,
    pub pet_id: String,
    pub doctor_id: String,
    #[serde(
        serialize_with = "crate::serde_helpers::serialize_datetime",
        deserialize_with = "crate::serde_helpers::deserialize_datetime"
    )]
    pub scheduled_at: DateTime<Utc>,
    pub status: CheckStatus,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub cost: Option<f32>,
    #[serde(
        serialize_with = "crate::serde_helpers::serialize_datetime",
        deserialize_with = "crate::serde_helpers::deserialize_datetime"
    )]
    pub created_at: DateTime<Utc>,
    #[serde(
        serialize_with = "crate::serde_helpers::serialize_datetime",
        deserialize_with = "crate::serde_helpers::deserialize_datetime"
    )]
    pub updated_at: DateTime<Utc>,
}

impl HealthCheck {
    pub fn new(pet_id: String, doctor_id: String, scheduled_at: DateTime<Utc>) -> Result<Self> {
        if scheduled_at < Utc::now() {
            return Err(CoreError::InvalidDate(
                "Cannot schedule check in the past".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: None,
            pet_id,
            doctor_id,
            scheduled_at,
            status: CheckStatus::Scheduled,
            diagnosis: None,
            treatment: None,
            notes: None,
            cost: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        if self.status != CheckStatus::Scheduled {
            return Err(CoreError::ValidationError(
                "Can only start a scheduled check".to_string(),
            ));
        }
        self.status = CheckStatus::InProgress;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn complete(
        &mut self,
        diagnosis: String,
        treatment: Option<String>,
        cost: Option<f32>,
    ) -> Result<()> {
        if self.status != CheckStatus::InProgress {
            return Err(CoreError::ValidationError(
                "Can only complete an in-progress check".to_string(),
            ));
        }

        if let Some(c) = cost {
            if c < 0.0 {
                return Err(CoreError::ValidationError(
                    "Cost cannot be negative".to_string(),
                ));
            }
        }

        self.diagnosis = Some(diagnosis);
        self.treatment = treatment;
        self.cost = cost;
        self.status = CheckStatus::Completed;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        if self.status == CheckStatus::Completed {
            return Err(CoreError::ValidationError(
                "Cannot cancel a completed check".to_string(),
            ));
        }
        self.status = CheckStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_notes(&mut self, notes: String) {
        self.notes = Some(notes);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_health_check() {
        let pet_id = "pets:test123".to_string();
        let doctor_id = "doctors:test456".to_string();
        let scheduled = Utc::now() + Duration::days(1);

        let check = HealthCheck::new(pet_id.clone(), doctor_id.clone(), scheduled).unwrap();

        assert_eq!(check.pet_id, pet_id);
        assert_eq!(check.doctor_id, doctor_id);
        assert_eq!(check.status, CheckStatus::Scheduled);
    }

    #[test]
    fn test_cannot_schedule_in_past() {
        let pet_id = "pets:test123".to_string();
        let doctor_id = "doctors:test456".to_string();
        let past = Utc::now() - Duration::days(1);

        let result = HealthCheck::new(pet_id, doctor_id, past);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_workflow() {
        let pet_id = "pets:test123".to_string();
        let doctor_id = "doctors:test456".to_string();
        let scheduled = Utc::now() + Duration::hours(2);

        let mut check = HealthCheck::new(pet_id, doctor_id, scheduled).unwrap();

        // Start the check
        check.start().unwrap();
        assert_eq!(check.status, CheckStatus::InProgress);

        // Complete the check
        check
            .complete(
                "Healthy".to_string(),
                Some("Vaccination".to_string()),
                Some(50.0),
            )
            .unwrap();
        assert_eq!(check.status, CheckStatus::Completed);
        assert_eq!(check.diagnosis, Some("Healthy".to_string()));
        assert_eq!(check.cost, Some(50.0));
    }

    #[test]
    fn test_cancel_check() {
        let pet_id = "pets:test123".to_string();
        let doctor_id = "doctors:test456".to_string();
        let scheduled = Utc::now() + Duration::hours(3);

        let mut check = HealthCheck::new(pet_id, doctor_id, scheduled).unwrap();
        check.cancel().unwrap();
        assert_eq!(check.status, CheckStatus::Cancelled);
    }

    #[test]
    fn test_cannot_cancel_completed() {
        let pet_id = "pets:test123".to_string();
        let doctor_id = "doctors:test456".to_string();
        let scheduled = Utc::now() + Duration::hours(1);

        let mut check = HealthCheck::new(pet_id, doctor_id, scheduled).unwrap();
        check.start().unwrap();
        check
            .complete("Diagnosis".to_string(), None, None)
            .unwrap();

        let result = check.cancel();
        assert!(result.is_err());
    }
}
