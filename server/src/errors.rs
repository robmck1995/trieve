use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use serde::{Deserialize, Serialize};
use std::convert::From;
use utoipa::ToSchema;
use uuid::Error as ParseError;

#[derive(Serialize, Deserialize, Debug, Display, derive_more::Error)]
#[display(fmt = "{}", message)]
pub struct DefaultError {
    pub message: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Display, ToSchema)]
pub struct ErrorResponseBody {
    pub message: String,
}

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error: {_0}")]
    InternalServerError(String),

    #[display(fmt = "BadRequest: {_0}")]
    BadRequest(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Forbidden")]
    Forbidden,

    #[display(fmt = "Not Found")]
    NotFound,
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError(ref message) => HttpResponse::InternalServerError()
                .json(ErrorResponseBody {
                    message: message.to_string(),
                }),
            ServiceError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(ErrorResponseBody {
                    message: message.to_string(),
                })
            }
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
            ServiceError::Forbidden => HttpResponse::Forbidden().json("Forbidden"),
            ServiceError::NotFound => HttpResponse::NotFound().json("Record not found"),
        }
    }
}

// we can return early in our handlers if UUID provided by the user is not valid
// and provide a custom message
impl From<ParseError> for ServiceError {
    fn from(_: ParseError) -> ServiceError {
        ServiceError::BadRequest("Invalid UUID".into())
    }
}

impl From<DBError> for ServiceError {
    fn from(error: DBError) -> ServiceError {
        // Right now we just care about UniqueViolation from diesel
        // But this would be helpful to easily map errors as our app grows
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return ServiceError::BadRequest(message);
                }
                ServiceError::InternalServerError("Unknown DB Error. Please try again later".into())
            }
            _ => ServiceError::InternalServerError(
                "Internal Server Error. Please try again later".into(),
            ),
        }
    }
}
