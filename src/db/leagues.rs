use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::MyError,
    models::{
        Authorization, Division, League, MiniLeague, MiniTeam, MiniUser, Team, TeamDivAssociation,
    },
};

pub async fn get_teams_with_divisionid(
    client: &Client,
    divisionid: i64,
) -> Result<Vec<TeamDivAssociation>, MyError> {
    let _stmt = "SELECT $table_fields FROM teamDivAssociations WHERE divisionid=$1";
    let _stmt = _stmt.replace("$table_fields", &Team::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&divisionid])
        .await?
        .iter()
        .map(|row| TeamDivAssociation::from_row_ref(row).unwrap())
        .collect::<Vec<TeamDivAssociation>>();
    Ok(results)
}

pub async fn get_divs_for_league_id(
    client: &Client,
    leagueid: i64,
) -> Result<Vec<Division>, MyError> {
    let _stmt = "SELECT $table_fields FROM divisions WHERE leagueid=$1;";
    let _stmt = _stmt.replace("$table_fields", &Division::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&leagueid])
        .await?
        .iter()
        .map(|row| Division::from_row_ref(row).unwrap())
        .collect::<Vec<Division>>();

    Ok(results)
}

pub async fn get_leagues(client: &Client) -> Result<Vec<League>, MyError> {
    let _stmt = "SELECT $table_fields FROM leagues;";
    let _stmt = _stmt.replace("$table_fields", &League::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[])
        .await?
        .iter()
        .map(|row| League::from_row_ref(row).unwrap())
        .collect::<Vec<League>>();
    Ok(results)
}

pub async fn get_league_from_id(client: &Client, leagueid: i64) -> Result<League, MyError> {
    log::debug!("Getting league {leagueid}");
    let _stmt = "SELECT $table_fields FROM leagues WHERE id=$1;";
    let _stmt = _stmt.replace("$table_fields", &League::sql_table_fields());
    log::debug!("Running this statement: {0}", _stmt);
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&leagueid])
        .await?
        .iter()
        .map(|row| League::from_row_ref(row).unwrap())
        .collect::<Vec<League>>()
        .pop()
        .ok_or(MyError::NotFound)?;

    Ok(results)
}
