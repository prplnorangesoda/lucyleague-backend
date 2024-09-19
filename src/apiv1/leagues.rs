use crate::db;
use crate::errors::MyError;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Client;

use crate::apiv1::LeagueResponse;
use crate::AppState;

#[get("/api/v1/leagues/{league_id}")]
pub async fn get_league(
    state: web::Data<AppState>,
    league_id: web::Path<i64>,
) -> Result<HttpResponse, Error> {
    log::debug!("GET request at /api/v1/leagues/league_id");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let league_info = db::get_league_from_id(&client, *league_id).await?;

    let teams = db::get_teams_with_leagueid(&client, *league_id).await?;

    let results = LeagueResponse {
        info: league_info,
        teams,
    };
    Ok(HttpResponse::Ok().json(results))
}
