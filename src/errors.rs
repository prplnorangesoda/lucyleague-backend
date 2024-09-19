// Errors that our crate may throw.
use actix_web::{HttpResponse, ResponseError};
use deadpool_postgres::PoolError;
use derive_more::{Display, Error, From};
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;

use crate::steamapi::ApiError;

#[derive(Debug, Display, Error, From)]
pub enum MyError {
    NotFound,
    ExternalApiError(ApiError),
    PGError(PGError),
    PGMError(PGMError),
    PoolError(PoolError),
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            MyError::NotFound => HttpResponse::NotFound().finish(),
            MyError::PoolError(ref err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
