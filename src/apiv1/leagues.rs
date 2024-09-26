use crate::db;
use crate::errors::MyError;
use actix_web::body::MessageBody;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};

use crate::apiv1::grab_pool;
use crate::models::{Division, League, Team, TeamDivAssociation};
use crate::AppState;

#[get("/api/v1/leagues")]
async fn get_all_leagues(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    log::info!("GET request at /api/v1/leagues");
    let client = grab_pool(&state).await?;

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
    admins: Vec<AdminInfo>,
    teams: Option<Vec<TeamDivAssociation>>,
}

#[derive(Serialize, Deserialize)]
struct LeagueReturn {
    info: League,
    divisions: Vec<DivisionOptional>,
}

#[get("/api/v1/leagues/{league_id}")]
pub async fn get_league(
    state: web::Data<AppState>,
    league_id: web::Path<i64>,
) -> Result<HttpResponse, Error> {
    log::debug!("GET request at /api/v1/leagues/league_id");

    let client: Client = grab_pool(&state).await?;

    let league_info = db::leagues::get_league_from_id(&client, *league_id).await?;

    let league_divs: Vec<Division> =
        db::leagues::get_divs_for_league_id(&client, *league_id).await?;

    let mut divisions: Vec<DivisionOptional> = Vec::with_capacity(league_divs.len());
    for div in league_divs {
        let _admins = db::divisions::get_admins_for_div_id(&client, div.id).await?;
        let mut admins = Vec::with_capacity(_admins.len());
        for admin in _admins.into_iter() {
            admins.push(AdminInfo {
                inner_user_id: admin.id,
                relation: admin.relation,
            })
        }
        let teams = db::divisions::get_teams_for_div_id(&client, div.id).await?;
        divisions.push(DivisionOptional {
            info: div,
            admins,
            teams: Some(teams),
        })
    }

    let resp = LeagueReturn {
        info: league_info,
        divisions,
    };
    Ok(HttpResponse::Ok().json(resp))
}