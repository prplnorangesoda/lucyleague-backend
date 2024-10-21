//! The user facing API.
//! Authorization required endpoints are at the module [admin].

use crate::authorization;
use crate::authorization::get_authorization_for_user;
use crate::db;
use crate::errors::MyError;
use crate::models::League;
use crate::models::MiniTeam;
use crate::models::User;
use crate::steamapi;
use crate::CurrentHost;
use actix_web::{get, post, web, Error, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use deadpool_postgres::{Client, Pool};
use serde::Deserialize;
use serde::Serialize;

use std::collections::HashMap;

pub mod admin;
pub mod leagues;
pub mod login;
pub mod teams;
pub mod users;

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
    pub current_host: CurrentHost,
    pub pool: Pool,
    pub steam_auth_url: String,
    pub steam_api_key: String,
    pub root_user_steamid: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct LogoutFields {
    pub auth_token: String,
}

#[post("/api/v1/logout")]
pub async fn logout(
    body: web::Json<LogoutFields>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client = grab_pool(&state).await?;
    let user = match db::get_user_from_auth_token(&client, &body.auth_token).await {
        Ok(user) => user,
        Err(err) => return Ok(HttpResponse::BadRequest().body(format!("{err:?}"))),
    };

    db::revoke_user_authorization(&client, &user).await?;
    Ok(HttpResponse::Ok().finish())
}

/// All parameters that a valid openid request should have.

//https://rgl.gg/Login/Default.aspx?push=1&r=40
//&dnoa.userSuppliedIdentifier=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2F
//&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
//&openid.mode=id_res
//&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
//&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
//&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
//&openid.return_to=https%3A%2F%2Frgl.gg%2FLogin%2FDefault.aspx%3Fpush%3D1%26r%3D40%26dnoa.userSuppliedIdentifier%3Dhttps%253A%252F%252Fsteamcommunity.com%252Fopenid%252F
//&openid.response_nonce=2024-07-27T16%3A07%3A06Zdg9%2BzW7ALLLycjtF7T7mWe3qKp0%3D
//&openid.assoc_handle=34321234
//&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle
//&openid.sig=f9dFKCcwpaGUWp2VsXwMV7csgsU%3D */
const OPENID_NECESSARY_PARAMETERS: &[&str] = &[
    "openid.ns",
    "openid.mode",
    "openid.op_endpoint",
    "openid.claimed_id",
    "openid.identity",
    "openid.return_to",
    "openid.response_nonce",
    "openid.assoc_handle",
    "openid.signed",
    "openid.sig",
];

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct OpenIdFields {
    pub openid__ns: String,
    pub openid__mode: String,
    pub openid__claimed_id: String,
    pub openid__signed: String,
    pub openid__response_nonce: String,
    pub openid__op_endpoint: String,
    pub openid__identity: String,
    pub openid__return_to: String,
    pub openid__assoc_handle: String,
    pub openid__sig: String,
}

#[derive(Serialize, Deserialize)]
struct IsOpenIdValid {
    pub valid: bool,
    pub token_info: Option<Token>,
}

#[derive(Serialize, Deserialize)]
struct Token {
    pub token: String,
    pub expires: DateTime<Utc>,
}

pub async fn grab_pool(state: &AppState) -> Result<Client, MyError> {
    state.pool.get().await.map_err(MyError::PoolError)
}

#[post("/api/v1/verifylogin")]
pub async fn verify_openid_login(
    body: web::Json<OpenIdFields>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    log::info!("POST at /api/v1/loginverify");
    let encode = serde_json::to_string(&body.0).unwrap();
    let is_valid = match steamapi::verify_auth_underscores(&encode).await {
        Ok(is_valid) => is_valid,
        Err(_) => {
            log::debug!("There was an error reaching out to Steam");
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    if !is_valid {
        return Ok(HttpResponse::BadRequest().json(IsOpenIdValid {
            valid: false,
            token_info: None,
        }));
    }

    let map: HashMap<String, String> = serde_json::from_str(&encode).unwrap();
    // is valid, return a token
    let openid_identity: &String = match map.get("openid__identity") {
        Some(str) => str,
        None => {
            log::debug!("There was an error reaching out to Steam");
            return Ok(HttpResponse::BadRequest().body("Malformed request"));
        }
    };

    // let openid_sig = inner.get("openid.sig").expect("No openid.sig on request");
    let steamid = openid_identity.replace("https://steamcommunity.com/openid/id/", "");
    log::info!("Openid landing received from steamid: {steamid}");
    let client: Client = grab_pool(&state).await?;

    let auth = match db::get_user_from_steamid(&client, &steamid).await {
        // there is a user corresponding
        Ok(user) => {
            log::trace!("User found for steamid {steamid}");
            match crate::authorization::get_authorization_for_user(&client, &user).await {
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
            let user: User = match users::add_user_with_steamid(&state, &client, &steamid).await {
                Ok(user) => user,
                Err(error_whatever) => {
                    return Ok(HttpResponse::InternalServerError()
                        .body(format!("Error: {error_whatever:?}")))
                }
            };
            crate::db::get_authorization_for_user(&client, &user).await?
        }
    };
    Ok(HttpResponse::Ok().json(IsOpenIdValid {
        valid: true,
        token_info: Some(Token {
            token: auth.token,
            expires: auth.expires,
        }),
    }))
}

#[derive(Serialize, Deserialize)]
struct TeamDivResponse {
    pub id: i64,
    pub divisionid: i64,
    pub team_name: String,
    pub players: Vec<User>,
}
