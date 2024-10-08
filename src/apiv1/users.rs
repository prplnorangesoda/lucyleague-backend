use crate::db;
use crate::errors::MyError;
use crate::models::MiniUser;
use crate::models::User;
use crate::steamapi;
use crate::PlayerSummaryAccess;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Client;
use serde::Deserialize;
use serde::Serialize;
use std::num::NonZeroU32;

use crate::AppState;

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

#[derive(Serialize, Deserialize)]
struct PageRequest {
    page: Option<u32>,
    amount_per_page: Option<std::num::NonZero<u32>>,
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    total_count: i64,
    page: u32,
    amount_per_page: std::num::NonZero<u32>,
    users: Vec<User>,
}
#[get("/api/v1/users")]
pub async fn get_users_paged(
    state: web::Data<AppState>,
    query: web::Query<PageRequest>,
) -> Result<HttpResponse, Error> {
    let client = state.pool.get().await.unwrap();
    let amount = query
        .amount_per_page
        .unwrap_or(NonZeroU32::new(10).unwrap());
    let page = query.page.unwrap_or(0);

    let total_count = db::get_user_count(&client);
    let users = db::get_user_page(&client, page, amount);

    let joined = futures::future::try_join(total_count, users).await?;

    Ok(HttpResponse::Ok().json(UserResponse {
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
) -> Result<HttpResponse, Error> {
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
