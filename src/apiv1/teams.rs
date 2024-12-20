use crate::db;
use crate::db::get_user_from_auth_token;
use crate::errors::MyError;
use crate::grab_pool;
use crate::models::{League, MiniTeam, Team, TeamDivAssociation, User};
use crate::permission::UserPermission;
use crate::steamapi;
use crate::CurrentHost;
use actix_web::{delete, get, post, web, Error, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
use serde::Serialize;

use crate::admin::AuthHeader;
use crate::apiv1::DeepTeamDivResponse;
use crate::AppState;

use super::HttpResult;

#[derive(Serialize, Deserialize)]
struct TeamReturn {
    pub info: Team,
    pub owner: User,
    pub team_div_assocs: Vec<TeamDivAssociation>,
}
#[get("/api/v1/teams/{team_id}")]
pub async fn get_team(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResult {
    log::info!("GET /api/v1/teams/{path}");
    let team_id = path.into_inner();
    if team_id < 0 {
        return Err(MyError::NotFound.into());
    }

    let client = grab_pool(&state).await?;

    let team = db::get_team_from_id(&client, team_id).await?;
    let (owner, team_div_assocs) = futures::try_join!(
        db::get_user_from_internal_id(&client, team.owner_id),
        db::team_div_assocs::get_team_tdas_teamid(&client, team_id)
    )?;
    let resp = TeamReturn {
        info: team,
        owner,
        team_div_assocs,
    };

    Ok(HttpResponse::Ok().json(resp))
}

// #[delete("/api/v1/teamdivassocs/{team_id}")]
// pub async fn del_tda(
//      _state: web::Data<AppState>,
//      _path: web::Path<i64>,
//      _auth_header: web::Header<AuthHeader>,
// ) -> HttpResult {
//     log::debug!("OK!");
//     Ok(HttpResponse::Ok().body("example"))
// }

#[get("/api/v1/teamdivassocs/{team_id}")]
pub async fn get_tda(state: web::Data<AppState>, path: web::Path<i64>) -> HttpResult {
    log::info!("GET /api/v1/teamdivassocs/{path}");
    let team_div_assoc_id = path.into_inner();
    if team_div_assoc_id < 0 {
        return Err(MyError::NotFound.into());
    }
    log::debug!("Grabbing pool");
    let client = grab_pool(&state).await?;
    log::debug!("Getting teamdivassociation");
    let team_div_assoc: TeamDivAssociation =
        db::get_teamdivassociation_from_id(&client, team_div_assoc_id).await?;
    log::debug!("Getting team");

    let team = db::get_team_from_id(&client, team_div_assoc.teamid)
        .await
        .expect("should be able to get team from id");

    let players = db::get_team_players(&client, &team_div_assoc)
        .await
        .expect("should be able to get team players");

    let mut current_players = Vec::with_capacity(players.len());
    let mut past_players = Vec::with_capacity(players.len());

    for player in players.into_iter() {
        if player.assoc.ended_at.is_some() {
            past_players.push(player)
        } else {
            current_players.push(player)
        }
    }

    let resp = DeepTeamDivResponse {
        association_info: team_div_assoc,
        team_info: team,
        current_players,
        past_players,
    };
    Ok(HttpResponse::Ok().json(resp))
}
#[derive(Serialize, Deserialize)]
struct TeamInfo {
    pub team_name: String,
    pub team_tag: String,
}
#[post("/api/v1/teams")]
pub async fn post_team(
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
