use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::errors::AppError;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.body_text()))?;

        value
            .validate()
            .map_err(|e| AppError::BadRequest(format_validation_errors(&e)))?;

        Ok(ValidatedJson(value))
    }
}

fn format_validation_errors(errors: &validator::ValidationErrors) -> String {
    let field_errors = errors.field_errors();
    let mut fields: Vec<_> = field_errors.iter().collect();
    fields.sort_by_key(|(name, _)| *name);

    fields
        .into_iter()
        .map(|(field, errs)| {
            let messages: Vec<String> = errs
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| e.code.to_string())
                })
                .collect();
            format!("{}: {}", field, messages.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Validate)]
    struct TestStruct {
        #[validate(length(min = 1, message = "must not be empty"))]
        name: String,
    }

    #[test]
    fn formats_single_field_error() {
        let s = TestStruct {
            name: "".to_string(),
        };
        let err = s.validate().unwrap_err();
        let msg = format_validation_errors(&err);
        assert!(msg.contains("name"));
        assert!(msg.contains("must not be empty"));
    }

    #[test]
    fn valid_struct_passes() {
        let s = TestStruct {
            name: "hello".to_string(),
        };
        assert!(s.validate().is_ok());
    }
}
