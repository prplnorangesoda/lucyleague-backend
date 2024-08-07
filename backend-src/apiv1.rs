use crate::models::MiniLeague;
// Main execution and routes.
use crate::steamapi;
use crate::PlayerSummaryAccess;
use crate::db;
use crate::models::User;
use crate::errors::MyError;
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use crate::authorization::get_authorization_for_user;
use deadpool_postgres::{Client, Pool};
use crate::models::MiniUser;
use crate::models::League;
use crate::models::Team;
use std::collections::HashMap;
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
}
#[derive(serde::Serialize, serde::Deserialize)]
struct UserResponse {
    user: User,
    teams: Vec<Team>
}
#[derive(serde::Serialize, serde::Deserialize)]
struct LeagueResponse {
    info: League,
    teams: Vec<Team>
}

#[get("/api/v1/leagues/{league_id}")]
pub async fn get_league(
    state: web::Data<AppState>,
    league_id: web::Path<i64>    
) -> Result<HttpResponse, Error> {
    println!("GET request at /api/v1/leagues/league_id");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let league_info = db::get_league(&client, *league_id).await?;

    let teams = db::get_teams_with_leagueid(&client, *league_id).await?;

    let results = LeagueResponse {
        info: league_info,
        teams
    };
    Ok(HttpResponse::Ok().json(results))
}

#[post("/api/v1/leagues")]
pub async fn post_league(
    league: web::Json<MiniLeague>,
    state: web::Data<AppState>
) -> Result<HttpResponse, Error> {
    println!("POST request at /api/v1/leagues");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;
    let league = league.into_inner();
    let response = db::add_league(&client, league).await?;

    Ok(HttpResponse::Ok().json(response))
}

#[get("/login/landing")]
pub async fn openid_landing(
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    println!("GET request at login/landing");
    let inner = query.into_inner();
    println!("{inner:?}");

    match steamapi::verify_authentication_with_steam(&inner).await {
        Ok(yeah) => {match yeah {
            true => {},
            false => {
                return Ok(HttpResponse::BadRequest().body("Could not verify your identity with Steam"))
            }
        }},
        Err(some) => {
            return Ok(HttpResponse::InternalServerError().body(some.to_string()))
        }
    }


    let openid_identity: &String =  match inner.get("openid.identity") {
            Some(str) => str,
            None => return Ok(HttpResponse::BadRequest().finish())
        };

    // let openid_sig = inner.get("openid.sig").expect("No openid.sig on request");
    let steamid = openid_identity.replace("https://steamcommunity.com/openid/id/", "");
    println!("Openid landing received from steamid: {steamid}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let auth = match db::get_user_from_steamid(&client, &steamid).await {
        // there is a user corresponding
        Ok(user) => match get_authorization_for_user(&client, &user).await {
            Ok(auth) => auth,
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().body("500 Internal Server Error"))
            }
        },
        // user wasn't found
        Err(_) => {
            let user: User =  match add_user_with_steamid(&state, &client, &steamid).await
                {
                    Ok(user) => user,
                    Err(_) => return Ok(HttpResponse::InternalServerError().finish())
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
        .append_header(("Location", "/home"))
        .body(format!("{auth:?}")))
}


#[get("/api/v1/user/steamid/{steamid}")]
pub async fn get_user_from_steamid(
    state: web::Data<AppState>,
    steamid: web::Path<String>,
) -> Result<HttpResponse, Error> {
    println!("GET request at /api/user/steamid/{steamid}");
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
    println!("GET request at /api/user/authtoken/{authtoken}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let user = db::get_user_from_auth_token(&client, &authtoken).await?;

    Ok(HttpResponse::Ok().json(user))
}

pub async fn get_users(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    println!("GET request at /users");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let users = db::get_users(&client).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn add_user(
    user: web::Json<MiniUser>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let user_info = user.into_inner();
    println!(
        "creating user with steamid: {0}, username: {1}",
        &user_info.steamid, &user_info.username
    );

    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let new_user = db::add_user(&client, user_info).await?;

    Ok(HttpResponse::Ok().json(new_user))
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

    db::add_user(db_client, user).await
}

#[get("/api/v1/leagues")]
async fn get_all_leagues(state: web::Data<AppState>) -> Result <HttpResponse, Error> {
    println!("GET request at /api/v1/leagues");
    let client = state.pool.get().await.map_err(MyError::PoolError)?;
    
    let leagues: Vec<League> = db::get_leagues(&client).await?;


    Ok(HttpResponse::Ok().json(leagues)) 
}


#[get("/api/v1/teams/{team_id}")]
async fn get_team(path: web::Path<u32>) -> impl Responder {
    println!("GET request at /teams/id");
    let team_id = path.into_inner();
    println!("Getting info for team id {team_id}");
    if team_id != 3 {
        return HttpResponse::NotFound().body("Team id not found");
    }
    HttpResponse::Ok().body(format!("Team {team_id}"))
}

#[get("/login")]
async fn get_openid(data: web::Data<AppState>) -> impl Responder {
    println!("GET request at /login");
    HttpResponse::Found()
        .insert_header(("Location", data.into_inner().steam_auth_url.clone()))
        .body("Redirecting...")
}




