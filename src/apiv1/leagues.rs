use crate::db;
use crate::errors::MyError;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};

use crate::models::{Division, League, Team};
use crate::AppState;

#[get("/api/v1/leagues")]
async fn get_all_leagues(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/leagues");
    let client = state.pool.get().await.map_err(MyError::PoolError)?;

    let leagues: Vec<League> = db::leagues::get_leagues(&client).await?;

    Ok(HttpResponse::Ok().json(leagues))
}

#[derive(Serialize, Deserialize)]
struct AdminInfo {
    inner_user_id: i64,
    relation: String,
}

#[derive(Serialize, Deserialize)]
struct DivisionOptional {
    info: Division,
    teams: Option<Vec<Team>>,
}

#[derive(Serialize, Deserialize)]
struct LeagueReturn {
    info: League,
    admins: Vec<AdminInfo>,
    divisions: Vec<DivisionOptional>,
}

#[get("/api/v1/leagues/{league_id}")]
pub async fn get_league(
    state: web::Data<AppState>,
    league_id: web::Path<i64>,
) -> Result<HttpResponse, Error> {
    log::debug!("GET request at /api/v1/leagues/league_id");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let league_info = db::leagues::get_league_from_id(&client, *league_id).await?;

    Ok(HttpResponse::Ok().json(league_info))
}
