use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    authorization::create_authorization_for_user,
    errors::MyError,
    models::{
        Authorization, League, MiniLeague, MiniTeam, MiniUser, Team, TeamDivAssociation, User,
        UserTeam,
    },
    permission::UserPermission,
};

pub async fn get_team_tdas_teamid(
    client: &Client,
    teamid: i64,
) -> Result<Vec<TeamDivAssociation>, MyError> {
    let sql_string = "SELECT $tda_f \
        FROM teamDivAssociations \
        WHERE teamid=$1"
        .replace("$tda_f", &TeamDivAssociation::sql_table_fields());

    let stmt = client.prepare(&sql_string).await?;

    let rows = client
        .query(&stmt, &[&teamid])
        .await?
        .iter()
        .map(TeamDivAssociation::from_row_ref)
        .map(Result::unwrap)
        .collect();

    Ok(rows)
}
pub async fn get_team_div_assoc_from_id(
    client: &Client,
    id: i64,
) -> Result<TeamDivAssociation, MyError> {
    let sql_string = "SELECT $table_fields \
      FROM teamDivAssociations \
      WHERE id=$1"
        .replace("$table_fields", &TeamDivAssociation::sql_table_fields());

    let stmt = client.prepare(&sql_string).await?;

    let row = client.query_one(&stmt, &[&id]).await?;

    Ok(TeamDivAssociation::from_row(row).unwrap())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MiniTeamDivAssociation {
    pub roster_name: Option<String>,
    pub teamid: i64,
    pub divisionid: i64,
    pub is_private: bool,
}
pub async fn add_team_div_assoc(
    client: &Client,
    teamdiv: MiniTeamDivAssociation,
) -> Result<TeamDivAssociation, MyError> {
    let sql_string = "INSERT INTO \
      teamDivAssociations (roster_name, teamid, divisionid, created_at, is_private ) \
      VALUES($1, $2, $3, $4, $5) \
      RETURNING $table_fields"
        .replace("$table_fields", &TeamDivAssociation::sql_table_fields());

    let stmt = client.prepare(&sql_string).await?;

    let row = client
        .query_one(
            &stmt,
            &[
                &teamdiv.roster_name,
                &teamdiv.teamid,
                &teamdiv.divisionid,
                &chrono::offset::Utc::now(),
                &teamdiv.is_private,
            ],
        )
        .await
        .map_err(|_| MyError::NotFound)?;

    Ok(TeamDivAssociation::from_row(row).unwrap())
}
