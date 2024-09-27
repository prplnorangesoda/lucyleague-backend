use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::MyError,
    models::{DivisionAdmin, Team, TeamDivAssociation},
};

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

pub async fn get_teams_for_div_id(
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
