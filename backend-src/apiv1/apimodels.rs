use serde::{Serialize, Deserialize};

use crate::models::*;

#[derive(Serialize, Deserialize)]
pub struct UserTeamBody {
    pub league_id: i64,
    pub user_steamids: Vec<String>,
    pub team_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
    pub teams: Vec<Team>,
}
#[derive(Serialize, Deserialize)]
pub struct LeagueResponse {
    pub info: League,
    pub teams: Vec<Team>,
}