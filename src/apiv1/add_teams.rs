use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::db::team_div_assocs::MiniTeamDivAssociation;
use crate::grab_pool;
use crate::models::TeamDivAssociation;
use crate::permission::UserPermission;
use crate::AppState;
use crate::{db, errors::MyError};
use chrono::{DateTime, Utc};

use super::HttpResult;

#[derive(Serialize, Deserialize)]
struct TeamDivAssocInfo {
    pub roster_name: Option<String>,
    pub teamid: i64,
    pub leagueid: i64,
    pub is_private: bool,
}
#[post("/api/v1/leagues/{id}/teams")]
pub async fn post_team_to_league(
    state: web::Data<AppState>,
    auth_header: web::Header<super::admin::AuthHeader>,
    new_team: web::Json<TeamDivAssocInfo>,
) -> HttpResult {
    let client = grab_pool(&state).await?;
    let token = auth_header.0 .0;

    let user = match db::get_user_from_auth_token(&client, &token).await {
        Ok(user) => user,
        Err(MyError::NotFound) => {
            return Ok(HttpResponse::Unauthorized().body("Invalid authorization token"))
        }
        Err(err) => return Ok(HttpResponse::InternalServerError().body(format!("{err:?}"))),
    };

    let leagueid = new_team.leagueid;
    let league = match db::leagues::get_league_from_id(&client, leagueid).await {
        Ok(league) => league,
        Err(MyError::NotFound) => {
            return Ok(HttpResponse::NotFound().body("League not found with id {leagueid}"))
        }
        Err(err) => return Ok(HttpResponse::InternalServerError().body(format!("{err:?}"))),
    };

    if !user.admin_or_perm(UserPermission::CreateTeam) && !league.accepting_teams {
        return Ok(HttpResponse::BadRequest().body("League not accepting new teams"));
    }

    // authorized to sign this team up, check if they actually own the specified team
    let team = match db::get_team_from_id(&client, new_team.teamid).await {
        Ok(team) => team,
        Err(MyError::NotFound) => return Ok(HttpResponse::BadRequest().body("Team not found")),
        Err(err) => return Ok(HttpResponse::InternalServerError().body(format!("{err:?}"))),
    };

    if user.id != team.owner_id {
        return Ok(HttpResponse::BadRequest().body("you don't own this team?"));
    }

    let divs = db::leagues::get_divs_for_league_id(&client, leagueid).await?;

    // get the div with the highest priority to add teams to
    let div = match divs.iter().reduce(|previous, current| {
        if current.prio > previous.prio {
            return current;
        } else {
            return previous;
        }
    }) {
        Some(div) => div,
        None => {
            return Ok(HttpResponse::BadRequest()
                .body("The specified league is not accepting new team registrations"))
        }
    };

    for roster in db::get_rosters_for_user_id(&client, user.id)
        .await?
        .into_iter()
    {
        if roster.team.association_info.divisionid == div.id {
            return Ok(HttpResponse::BadRequest()
                .body("You are currently signed up to a team in this league"));
        }
    }

    let final_assoc = MiniTeamDivAssociation {
        divisionid: div.id,
        teamid: team.id,
        is_private: new_team.is_private,
        roster_name: new_team.roster_name.clone(),
    };

    let assoc = db::team_div_assocs::add_team_div_assoc(&client, final_assoc).await?;

    db::teams::add_user_team_id(
        &client,
        user.id,
        assoc.id,
        db::teams::UserTeamAffiliation::Leader,
    )
    .await
    .expect("should be able to add user team id");

    Ok(HttpResponse::Created().json(assoc))
}
