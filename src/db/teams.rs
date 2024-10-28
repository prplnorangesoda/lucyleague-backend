use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
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
    teamdivid: i64,
    status: UserTeamAffiliation,
) -> Result<UserTeam, MyError> {
    let _stmt = "INSERT INTO \
    userTeamAssociation(userid, teamdivid, created_at, affiliation) \
    VALUES($1, $2, $3, $4) \
    RETURNING $table_fields"
        .replace("$table_fields", &UserTeam::sql_table_fields());

    let stmt = client.prepare(&_stmt).await?;

    let row = client
        .query_one(
            &stmt,
            &[
                &userid,
                &teamdivid,
                &chrono::offset::Utc::now(),
                &(status as i32),
            ],
        )
        .await
        .map_err(|_| MyError::NotFound)?;

    Ok(UserTeam::from_row(row).unwrap())
}
