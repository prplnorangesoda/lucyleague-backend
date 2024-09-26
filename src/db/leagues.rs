use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::MyError,
    models::{Authorization, League, MiniLeague, MiniTeam, MiniUser, Team, User, UserTeam},
    permission::UserPermission,
};
pub async fn get_teams_with_leagueid(client: &Client, leagueid: i64) -> Result<Vec<Team>, MyError> {
    let _stmt = "SELECT $table_fields FROM teams WHERE leagueid=$1";
    let _stmt = _stmt.replace("$table_fields", &Team::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&leagueid])
        .await?
        .iter()
        .map(|row| Team::from_row_ref(row).unwrap())
        .collect::<Vec<Team>>();
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
