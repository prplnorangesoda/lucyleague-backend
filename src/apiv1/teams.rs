use crate::db;
use crate::db::get_user_from_auth_token;
use crate::errors::MyError;
use crate::grab_pool;
use crate::models::{League, MiniTeam, Team, TeamDivAssociation, User};
use crate::permission::UserPermission;
use crate::steamapi;
use crate::CurrentHost;
use actix_web::{get, post, web, Error, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
use serde::Serialize;

use crate::apiv1::TeamDivResponse;
use crate::AppState;

use super::HttpResult;

#[derive(Serialize, Deserialize)]
struct TeamReturn {
    pub info: Team,
    pub owner: User,
    pub team_div_assocs: Vec<TeamDivAssociation>,
}
#[get("/api/v1/teams/{team_id}")]
async fn get_team(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResult {
    log::info!("GET /api/v1/teams/{path}");
    let team_id = path.into_inner();
    if team_id < 0 {
        return Err(MyError::NotFound.into());
    }

    let client = grab_pool(&state).await?;

    let team = db::get_team_from_id(&client, team_id).await?;
    let owner = db::get_user_from_internal_id(&client, team.owner_id).await?;

    let resp = TeamReturn {
        info: team,
        owner,
        team_div_assocs: Vec::new(),
    };

    Ok(HttpResponse::Ok().json(resp))
}
// this will be retroactively changed to be for a teamDivAssociation and not a root team
// maybe /rootteam/{team_id}?
#[get("/api/v1/teamdivassocs/{team_id}")]
async fn get_team_div_assoc(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResult {
    log::info!("GET /api/v1/teams/{path}");
    let team_div_assoc_id = path.into_inner();
    if team_div_assoc_id < 0 {
        return Err(MyError::NotFound.into());
    }
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let team_div_assoc: TeamDivAssociation =
        db::get_teamdivassociation_from_id(&client, team_div_assoc_id).await?;
    let team = db::get_team_from_id(&client, team_div_assoc.teamid).await?;

    let players = db::get_team_players(&client, &team_div_assoc).await?;
    let resp = TeamDivResponse {
        id: team.id,
        divisionid: team_div_assoc.divisionid,
        team_name: team.team_name,
        players,
    };
    Ok(HttpResponse::Ok().json(resp))
}
#[derive(Serialize, Deserialize)]
struct TeamInfo {
    pub team_name: String,
    pub team_tag: String,
}
#[post("/api/v1/teams")]
async fn post_team(
    state: web::Data<AppState>,
    auth_header: web::Header<super::admin::AuthHeader>,
    new_team: web::Json<TeamInfo>,
) -> HttpResult {
    log::info!("POST /api/v1/teams");
    let auth_token = auth_header.into_inner().0;
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let user = match get_user_from_auth_token(&client, &auth_token).await {
        Ok(u) => u,
        Err(MyError::NotFound) => {
            return Ok(HttpResponse::InternalServerError()
                .body(format!("User not found with token {auth_token}")))
        }
        Err(a) => {
            return Ok(HttpResponse::InternalServerError()
                .body(format!("An unhandled error occurred: {a:?}")))
        }
    };
    let team = new_team.into_inner();
    // let leagueid = team.leagueid;
    // let league = match db::leagues::get_league_from_id(&client, leagueid).await {
    //     Ok(league) => league,
    //     Err(_) => return Ok(HttpResponse::NotFound().body("League not found with id ${leagueid}")),
    // };

    // if !user.admin_or_perm(UserPermission::CreateTeam) && !league.accepting_teams {
    //     return Ok(HttpResponse::BadRequest().body("League not accepting new teams"));
    // }

    let team = db::add_team(
        &client,
        &MiniTeam {
            owner_id: user.id,
            team_name: team.team_name,
            team_tag: team.team_tag,
        },
    )
    .await?;

    // let resp = db::teams::add_user_team_id(
    //     &client,
    //     user.id,
    //     team.id,
    //     db::teams::UserTeamAffiliation::Leader,
    // )
    // .await?;
    Ok(HttpResponse::Created().json(team))
}
