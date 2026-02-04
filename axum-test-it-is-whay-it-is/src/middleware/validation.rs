use validator::Validate;

use crate::shared::error::{AppError, ValidationError};

/// Validate a struct using the validator crate
pub fn validate_request<T: Validate>(data: &T) -> Result<(), AppError> {
    data.validate().map_err(|errors| {
        let validation_errors: Vec<ValidationError> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| ValidationError {
                    field: field.to_string(),
                    message: error
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "Validation failed".to_string()),
                })
            })
            .collect();

        AppError::Validation(validation_errors)
    })
}
