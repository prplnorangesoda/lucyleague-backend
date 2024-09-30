use deadpool_postgres::Client;
use derive_more::derive::Debug;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::MyError,
    models::{DivisionAdmin, Team, TeamDivAssociation, WrappedDivisionAdmin},
};

pub async fn get_admins_for_div_id_wrapped(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<WrappedDivisionAdmin>, MyError> {
    let div_admins = get_admins_for_div_id(client, divisionid).await?;

    let stmt = client
        .prepare("SELECT users.username , users.avatarurl FROM users WHERE id=$1")
        .await?;

    let mut ret: Vec<WrappedDivisionAdmin> = Vec::new();
    for admin in div_admins.into_iter() {
        let row = client.query_one(&stmt, &[&admin.id]).await?;

        ret.push(WrappedDivisionAdmin {
            inner: admin,
            username: row.get("users.username"),
            avatarurl: row.get("users.avatarurl"),
        })
    }
    Ok(ret)
}
pub async fn get_admins_for_div_id(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<DivisionAdmin>, MyError> {
    let _stmt = "SELECT $table_fields FROM division_admins WHERE divisionid=$1";
    let _stmt = _stmt.replace("$table_fields", &DivisionAdmin::sql_table_fields());
    let stmt = client.prepare(&_stmt).await?;

    let results = client
        .query(&stmt, &[&divisionid])
        .await?
        .iter()
        .map(DivisionAdmin::from_row_ref)
        .map(Result::unwrap)
        .collect();

    Ok(results)
}

pub async fn get_teamassociations_for_div_id(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<TeamDivAssociation>, MyError> {
    let _stmt = "SELECT $table_fields FROM teamDivAssociations WHERE divisionid=$1";
    let _stmt = _stmt.replace("$table_fields", &TeamDivAssociation::sql_table_fields());
    let stmt = client.prepare(&_stmt).await?;

    let results = client
        .query(&stmt, &[&divisionid])
        .await?
        .iter()
        .map(TeamDivAssociation::from_row_ref)
        .map(Result::unwrap)
        .collect();

    Ok(results)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeepTeamDivAssociation {
    pub team_info: Team,
    pub association_info: TeamDivAssociation,
}
pub async fn get_teams_for_div_id(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<DeepTeamDivAssociation>, MyError> {
    let assocs = get_teamassociations_for_div_id(client, divisionid).await?;
    let _stmt = "SELECT $table_fields FROM teams WHERE id=$1"
        .replace("$table_fields", &Team::sql_table_fields());
    let stmt = client.prepare(&_stmt).await?;

    let mut ret: Vec<DeepTeamDivAssociation> = Vec::with_capacity(assocs.len());

    for assoc in assocs.into_iter() {
        let row = client.query_one(&stmt, &[&assoc.id]).await?;
        ret.push(DeepTeamDivAssociation {
            team_info: Team::from_row(row).unwrap(),
            association_info: assoc,
        })
    }
    Ok(ret)
}
