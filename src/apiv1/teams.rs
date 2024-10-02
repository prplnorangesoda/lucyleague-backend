use crate::db;
use crate::errors::MyError;
use crate::models::League;
use crate::models::MiniTeam;
use crate::models::TeamDivAssociation;
use crate::models::User;
use crate::steamapi;
use crate::CurrentHost;
use actix_web::{get, post, web, Error, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use deadpool_postgres::{Client, Pool};

use crate::apiv1::TeamResponse;
use crate::AppState;

// this will be retroactively changed to be for a teamDivAssociation and not a root team
// maybe /rootteam/{team_id}?
#[get("/api/v1/teams/{team_id}")]
async fn get_team(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/teams/{path}");
    let team_div_assoc_id = path.into_inner();
    if team_div_assoc_id < 0 {
        return Err(MyError::NotFound.into());
    }
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let team_div_assoc: TeamDivAssociation =
        db::get_teamdivassociation_from_id(&client, team_div_assoc_id).await?;
    let team = db::get_team_from_id(&client, team_div_assoc.teamid).await?;

    let players = db::get_team_players(&client, &team_div_assoc).await?;
    let resp = TeamResponse {
        id: team.id,
        divisionid: team_div_assoc.divisionid,
        team_name: team.team_name,
        players,
    };
    Ok(HttpResponse::Ok().json(resp))
}
#[post("/api/v1/teams")]
async fn post_team(
    state: web::Data<AppState>,
    new_team: web::Json<MiniTeam>,
) -> Result<HttpResponse, Error> {
    log::info!("POST request at /api/v1/teams");
    let team = new_team.into_inner();
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let leagueid = team.leagueid;
    let league = match db::leagues::get_league_from_id(&client, leagueid).await {
        Ok(league) => league,
        Err(_) => return Ok(HttpResponse::NotFound().body("League not found with id ${leagueid}")),
    };
    if !league.accepting_teams {
        return Ok(HttpResponse::BadRequest().body("League not accepting new teams"));
    }

    let resp = db::add_team(&client, &team).await?;
    Ok(HttpResponse::Created().json(resp))
}
