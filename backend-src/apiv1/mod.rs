// Main execution and routes.

use crate::authorization::get_authorization_for_user;
use crate::db;
use crate::errors::MyError;
use crate::models::League;
use crate::models::MiniTeam;
use crate::models::MiniUser;
use crate::models::User;
use crate::steamapi;
use crate::PlayerSummaryAccess;
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

pub mod admin;
mod apimodels;

use apimodels::*;

/*
https://rgl.gg/Login/Default.aspx?push=1&r=40
&dnoa.userSuppliedIdentifier=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2F
&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
&openid.mode=id_res
&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
&openid.return_to=https%3A%2F%2Frgl.gg%2FLogin%2FDefault.aspx%3Fpush%3D1%26r%3D40%26dnoa.userSuppliedIdentifier%3Dhttps%253A%252F%252Fsteamcommunity.com%252Fopenid%252F
&openid.response_nonce=2024-07-27T16%3A07%3A06Zdg9%2BzW7ALLLycjtF7T7mWe3qKp0%3D
&openid.assoc_handle=34321234
&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle
&openid.sig=f9dFKCcwpaGUWp2VsXwMV7csgsU%3D */
pub struct AppState {
    pub pool: Pool,
    pub steam_auth_url: String,
    pub steam_api_key: String,
    pub root_user_steamid: Option<String>,
}

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
/// All parameters that a valid openid request should have.
const OPENID_NECESSARY_PARAMETERS: &[&str] = &[
    "openid.ns",
    "openid.claimed_id",
    "openid.signed",
    "openid.response_nonce",
    "openid.op_endpoint",
];

#[get("/api/v1/login/landing")]
pub async fn openid_landing(
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    log::info!("GET request at /login/landing");
    let inner = query.into_inner();
    log::trace!("Query parameters: {inner:?}");

    for key in OPENID_NECESSARY_PARAMETERS {
        // because key is &&str, we have to dereference it to a pure &str
        // in order for it to not yell at us in compilation
        if !inner.contains_key(*key) {
            log::warn!("A malformed OpenId landing was received: {inner:?}");
            return Ok(HttpResponse::BadRequest()
                .body("Your openid landing was malformed in some way. Report this!"));
        }
    }

    match steamapi::verify_authentication_with_steam(&inner).await {
        Ok(yeah) => match yeah {
            true => {}
            false => {
                return Ok(
                    HttpResponse::BadRequest().body("Could not verify your identity with Steam")
                )
            }
        },
        Err(some) => return Ok(HttpResponse::InternalServerError().body(some.to_string())),
    }

    let openid_identity: &String = match inner.get("openid.identity") {
        Some(str) => str,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };

    // let openid_sig = inner.get("openid.sig").expect("No openid.sig on request");
    let steamid = openid_identity.replace("https://steamcommunity.com/openid/id/", "");
    log::info!("Openid landing received from steamid: {steamid}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let auth = match db::get_user_from_steamid(&client, &steamid).await {
        // there is a user corresponding
        Ok(user) => {
            log::trace!("User found for steamid {steamid}");
            match get_authorization_for_user(&client, &user).await {
                Ok(auth) => {
                    log::debug!("Assigning {auth:?} to {user:?}");
                    auth
                }
                Err(_) => {
                    log::error!("Internally failed to get authorization for {user:?}");
                    return Ok(
                        HttpResponse::InternalServerError().body("500 Internal Server Error")
                    );
                }
            }
        }
        // user wasn't found
        Err(_) => {
            log::info!("Creating a new user with steamid {steamid}");
            let user: User = match add_user_with_steamid(&state, &client, &steamid).await {
                Ok(user) => user,
                Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
            };
            get_authorization_for_user(&client, &user).await?
        }
    };
    Ok(HttpResponse::Found()
        .append_header((
            "Set-Cookie",
            format!(
                "auth-token={0}; Expires={1}; SameSite=Lax; Path=/",
                auth.token, auth.expires
            ),
        ))
        .append_header(("Location", "http://localhost:3000/home"))
        .finish())
}

#[get("/api/v1/user/steamid/{steamid}")]
pub async fn get_user_from_steamid(
    state: web::Data<AppState>,
    steamid: web::Path<String>,
) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/user/steamid/{steamid}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let user_res = db::get_user_from_steamid(&client, &steamid).await;

    match user_res {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => {
            if let MyError::NotFound = err {
                Ok(HttpResponse::NotFound().body("404 Not Found"))
            } else {
                Err(err.into())
            }
        }
    }
}

#[get("/api/v1/user/authtoken/{authtoken}")]
pub async fn get_user_from_auth_token(
    state: web::Data<AppState>,
    authtoken: web::Path<String>,
) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/user/authtoken/{authtoken}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let user = db::get_user_from_auth_token(&client, &authtoken).await?;

    Ok(HttpResponse::Ok().json(user))
}

pub async fn get_users(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/users");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let users = db::get_users(&client).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn add_user(
    user: web::Json<MiniUser>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user_info = user.into_inner();
    log::debug!(
        "creating user with steamid: {0}, username: {1}",
        &user_info.steamid,
        &user_info.username
    );

    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let new_user = db::add_user(&client, user_info).await?;

    Ok(HttpResponse::Created().json(new_user))
}

pub async fn add_user_with_steamid(
    state: &web::Data<AppState>,
    db_client: &Client,
    steamid: &str,
) -> Result<User, MyError> {
    let steam_user_access_level = steamapi::get_user_summary(&state.steam_api_key, steamid).await?;

    // hacky oneliner: extract public information regardless of return type
    let (PlayerSummaryAccess::All { public, .. } | PlayerSummaryAccess::Private { public }) =
        steam_user_access_level;
    let user = MiniUser {
        steamid: public.steamid,
        avatarurl: public.avatarfull,
        username: public.personaname,
        permissions: None,
    };

    let add_user_resp = db::add_user(db_client, user).await?;

    if let Some(rootid) = &state.root_user_steamid {
        if rootid == steamid {
            if let Ok(user) = db::set_super_user(db_client, &add_user_resp).await {
                log::info!("Set super user {user:?}")
            };
        }
    }
    Ok(add_user_resp)
}

#[get("/api/v1/leagues")]
async fn get_all_leagues(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/leagues");
    let client = state.pool.get().await.map_err(MyError::PoolError)?;

    let leagues: Vec<League> = db::get_leagues(&client).await?;

    Ok(HttpResponse::Ok().json(leagues))
}

#[derive(Serialize, Deserialize)]
struct TeamResponse {
    pub id: i64,
    pub leagueid: i64,
    pub team_name: String,
    pub players: Vec<User>,
}

#[get("/api/v1/teams/{team_id}")]
async fn get_team(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/teams/{path}");
    let team_id = path.into_inner();
    if team_id < 0 {
        return Err(MyError::NotFound.into());
    }
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    let team = db::get_team_from_id(&client, team_id).await?;
    let players = db::get_team_players(&client, &team).await?;
    let resp = TeamResponse {
        id: team.id,
        leagueid: team.leagueid,
        team_name: team.team_name,
        players,
    };
    Ok(HttpResponse::Created().json(resp))
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
    let league = match db::get_league_from_id(&client, leagueid).await {
        Ok(league) => league,
        Err(e) => return Ok(HttpResponse::NotFound().body("League not found with id ${leagueid}")),
    };
    if !league.accepting_teams {
        return Ok(HttpResponse::BadRequest().body("League not accepting new teams"));
    }

    let resp = db::add_team(&client, &team).await?;
    Ok(HttpResponse::Created().json(resp))
}

#[get("/api/v1/login")]
async fn get_openid(data: web::Data<AppState>) -> HttpResponse {
    log::info!("GET request at /login");
    HttpResponse::Found()
        .insert_header(("Location", data.into_inner().steam_auth_url.clone()))
        .body("Redirecting...")
}
