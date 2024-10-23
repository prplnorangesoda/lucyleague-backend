use crate::db;
use crate::errors::MyError;
use crate::models::MiniUser;
use crate::models::Team;
use crate::models::TeamDivAssociation;
use crate::models::User;
use crate::steamapi;
use crate::PlayerSummaryAccess;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Client;
use serde::Deserialize;
use serde::Serialize;
use std::num::NonZeroU32;

use crate::AppState;

#[derive(Serialize, Deserialize)]
struct UserParams {
    deep: Option<bool>,
}
#[derive(Serialize, Deserialize)]
struct UserResponse {
    info: User,
    ownerships: Option<Vec<Team>>,
    rosters: Option<Vec<TeamDivAssociation>>,
}
#[get("/api/v1/user/steamid/{steamid}")]
pub async fn get_user_from_steamid(
    state: web::Data<AppState>,
    steamid: web::Path<String>,
    query_params: web::Query<UserParams>,
) -> HttpResult {
    log::info!("GET request at /api/v1/user/steamid/{steamid}");
    let client: Client = crate::grab_pool(&state).await?;

    let user = match db::get_user_from_steamid(&client, &steamid).await {
        Ok(user) => user,
        Err(err) => {
            if let MyError::NotFound = err {
                return Ok(HttpResponse::NotFound().body("404 Not Found"));
            } else {
                return Err(err.into());
            }
        }
    };

    let rosters: Option<Vec<TeamDivAssociation>> = match query_params.deep {
        Some(deep) => {
            let mut resp = None;
            if deep {
                resp = Some(db::get_rosters_for_user_id(&client, user.id).await?)
            }
            resp
        }
        None => None,
    };
    let ownerships = match query_params.deep {
        Some(deep) => {
            let mut resp = None;
            if deep {
                resp = Some(db::get_ownerships_for_user_id(&client, user.id).await?)
            }
            resp
        }
        None => None,
    };

    let resp = UserResponse {
        info: user,
        rosters,
        ownerships,
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[get("/api/v1/user/authtoken/{authtoken}")]
pub async fn get_user_from_auth_token(
    state: web::Data<AppState>,
    authtoken: web::Path<String>,
    query_params: web::Query<UserParams>,
) -> HttpResult {
    log::info!("GET request at /api/v1/user/authtoken/{authtoken}");
    let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

    let user = db::get_user_from_auth_token(&client, &authtoken).await?;

    let rosters: Option<Vec<TeamDivAssociation>> = match query_params.deep {
        Some(deep) => {
            let mut resp = None;
            if deep {
                resp = Some(db::get_rosters_for_user_id(&client, user.id).await?)
            }
            resp
        }
        None => None,
    };
    let ownerships = match query_params.deep {
        Some(deep) => {
            let mut resp = None;
            if deep {
                resp = Some(db::get_ownerships_for_user_id(&client, user.id).await?)
            }
            resp
        }
        None => None,
    };

    let resp = UserResponse {
        info: user,
        rosters,
        ownerships,
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Serialize, Deserialize)]
struct PageRequest {
    page: Option<u32>,
    amount_per_page: Option<std::num::NonZero<u32>>,
}

#[derive(Serialize, Deserialize)]
struct PagedUserResponse {
    total_count: i64,
    page: u32,
    amount_per_page: std::num::NonZero<u32>,
    users: Vec<User>,
}
#[get("/api/v1/users")]
pub async fn get_users_paged(
    state: web::Data<AppState>,
    query: web::Query<PageRequest>,
) -> HttpResult {
    let client = state.pool.get().await.unwrap();
    let amount = query
        .amount_per_page
        .unwrap_or(NonZeroU32::new(10).unwrap());
    let page = query.page.unwrap_or(0);

    let total_count = db::get_user_count(&client);
    let users = db::get_user_page(&client, page, amount);

    let joined = futures::future::try_join(total_count, users).await?;

    Ok(HttpResponse::Ok().json(PagedUserResponse {
        total_count: joined.0,
        users: joined.1,
        page,
        amount_per_page: amount,
    }))
}

#[derive(Serialize, Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    amount_per_page: Option<std::num::NonZero<u32>>,
}

#[get("/api/v1/users/search")]
pub async fn search_users(
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> HttpResult {
    let client = state.pool.get().await.unwrap();
    let amount = query
        .amount_per_page
        .unwrap_or(NonZeroU32::new(10).unwrap());
    let page = query.page.unwrap_or(0);

    let results = db::search_usernames(&client, &query.q, page, amount).await?;

    Ok(HttpResponse::Ok().json(results))
}

pub async fn add_user_with_steamid(
    state: &web::Data<AppState>,
    db_client: &Client,
    steamid: &str,
) -> Result<User, MyError> {
    log::debug!("Adding users with steamid: {steamid}");
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

    log::trace!("Adding user in db");
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
