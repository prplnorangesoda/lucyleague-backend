use serde::{Deserialize, Serialize};

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
