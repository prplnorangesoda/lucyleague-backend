use crate::db;
use crate::db::divisions::DeepTeamDivAssociation;
use crate::errors::MyError;
use actix_web::body::MessageBody;
use actix_web::{get, web, Error, HttpResponse};
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};

use super::HttpResult;
use crate::apiv1::grab_pool;
use crate::models::{Division, League, Team, TeamDivAssociation, WrappedDivisionAdmin};
use crate::AppState;

#[get("/api/v1/leagues")]
async fn get_all_leagues(state: web::Data<AppState>) -> HttpResult {
    log::info!("GET /api/v1/leagues");
    let client = grab_pool(&state).await?;

    let leagues: Vec<League> = db::leagues::get_leagues(&client).await?;

    let mut league_responses: Vec<LeagueReturn> = Vec::with_capacity(leagues.len());

    for league in leagues {
        let league_divs: Vec<Division> =
            db::leagues::get_divs_for_league_id(&client, league.id).await?;

        let mut divisions: Vec<DivisionOptionalTeams> = Vec::with_capacity(league_divs.len());

        for div in league_divs {
            let admins = db::divisions::get_admins_for_div_id_wrapped(&client, div.id).await?;
            divisions.push(DivisionOptionalTeams {
                info: div,
                admins,
                teams: None,
            })
        }
        league_responses.push(LeagueReturn {
            divisions,
            info: league,
        })
    }

    Ok(HttpResponse::Ok().json(league_responses))
}

#[derive(Serialize, Deserialize)]
struct AdminInfo {
    inner_user_id: i64,
    relation: String,
}

#[derive(Serialize, Deserialize)]
struct DivisionOptionalTeams {
    info: Division,
    admins: Vec<WrappedDivisionAdmin>,
    teams: Option<Vec<DeepTeamDivAssociation>>,
}

#[derive(Serialize, Deserialize)]
struct LeagueReturn {
    info: League,
    divisions: Vec<DivisionOptionalTeams>,
}

#[get("/api/v1/leagues/{league_id}")]
pub async fn get_league(state: web::Data<AppState>, league_id: web::Path<i64>) -> HttpResult {
    log::info!("GET /api/v1/leagues/league_id");

    let client: Client = grab_pool(&state).await?;

    let league_info = db::leagues::get_league_from_id(&client, *league_id).await?;

    let league_divs: Vec<Division> =
        db::leagues::get_divs_for_league_id(&client, *league_id).await?;

    let mut divisions: Vec<DivisionOptionalTeams> = Vec::with_capacity(league_divs.len());
    for div in league_divs {
        let admins = db::divisions::get_admins_for_div_id_wrapped(&client, div.id)
            .await
            .expect("should be able to get admins");
        let teams = db::divisions::get_teams_for_div_id(&client, div.id)
            .await
            .expect("should be able to get teams");
        divisions.push(DivisionOptionalTeams {
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
