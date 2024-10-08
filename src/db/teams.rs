use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::ToSql;

use crate::errors::MyError;
use crate::models::UserTeam;

#[derive(Debug, Deserialize, Serialize)]
pub enum UserTeamAffiliation {
    Leader = 20,
    Officer = 10,
    Member = 0,
}

pub async fn add_user_team_id(
    client: &Client,
    userid: i64,
    teamid: i64,
    status: UserTeamAffiliation,
) -> Result<UserTeam, MyError> {
    todo!()
}
