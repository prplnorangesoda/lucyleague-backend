use actix_web::{post, HttpResponse};

use super::HttpResult;

#[post("/api/v1/leagues/{id}/teams")]
pub async fn post_team_to_league() -> HttpResult {
    Ok(HttpResponse::Ok().json({}))
}
