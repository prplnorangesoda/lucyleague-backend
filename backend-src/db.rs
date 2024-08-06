// Code that acts as a wrapper for database values.
use chrono::{DateTime, Utc};
use deadpool_postgres::{Client, GenericClient};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    authorization::create_authorization_for_user,
    errors::MyError,
    models::{Authorization, League, MiniLeague, MiniUser, Team, User},
};

pub async fn get_league(client: &Client, leagueid: i64) -> Result<League, MyError> {
    let _stmt = "SELECT $table_fields FROM leagues WHERE id=$1;";
    let _stmt = _stmt.replace("$table_fields", &League::sql_table_fields());
    println!("{0}", _stmt);
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

pub async fn add_league(client: &Client, league: MiniLeague) -> Result<League, MyError> {
    let _stmt = "INSERT INTO leagues(name) VALUES ($1) RETURNING $table_fields";
    let _stmt = _stmt.replace("$table_fields", &League::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&league.name])
        .await?
        .iter()
        .map(|row| League::from_row_ref(row).unwrap())
        .collect::<Vec<League>>()
        .pop()
        .unwrap();
    Ok(results)    
}
pub async fn get_user_from_auth_token(client: &Client, token: &str) -> Result<User, MyError> {
    let _stmt = include_str!("../sql/get_user_from_authtoken.sql");
    let stmt = client.prepare(&_stmt).await.unwrap();

    let received_auth = client
        .query(&stmt, &[&token])
        .await?
        .iter()
        .map(|row| Authorization::from_row_ref(row).unwrap())
        .collect::<Vec<Authorization>>()
        .pop()
        .ok_or(MyError::NotFound)?;

    let _stmt = "SELECT $table_fields FROM users WHERE id=$1;";
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client.query(&stmt, &[&received_auth.userid])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound)
}
pub async fn get_user_from_steamid(client: &Client, steamid: &str) -> Result<User, MyError> {
    let _stmt = include_str!("../sql/get_user_from_steamid.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(&stmt, &[&steamid])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound)
}

pub async fn get_authorization_for_user(
    client: &Client,
    user: &User,
) -> Result<Authorization, MyError> {
    let _stmt = include_str!("../sql/get_auth_token.sql");
    let _stmt = _stmt.replace("$fields", &Authorization::sql_fields());
    let stmt = client.prepare(&_stmt).await?;

    if let Ok(some) = client
        .query(&stmt, &[&user.id])
        .await?
        .iter()
        .map(|row| Authorization::from_row_ref(row).unwrap())
        .collect::<Vec<Authorization>>()
        .pop()
        .ok_or(MyError::NotFound)
    {
        Ok(some)
    } else {
        create_authorization_for_user(client, user).await
    }
}

pub async fn register_authorization(
    client: &Client,
    token: &str,
    user: &User,
    expiry: DateTime<Utc>,
) -> Result<Authorization, MyError> {
    let _stmt = include_str!("../sql/register_auth_token.sql");
    // $table_fields didn't work with this for some reason when i tested it
    let _stmt = _stmt.replace("$fields", &Authorization::sql_fields());
    println!("{}", &_stmt);
    println!("{}, {}", &user.id, &token);
    let stmt = client.prepare(&_stmt).await?;

    client
        .query(&stmt, &[&user.id, &token, &expiry])
        .await?
        .iter()
        .map(|row| Authorization::from_row_ref(row).unwrap())
        .collect::<Vec<Authorization>>()
        .pop()
        .ok_or(MyError::NotFound)
}

pub async fn get_users(client: &Client) -> Result<Vec<User>, MyError> {
    let sql_string = include_str!("../sql/get_users.sql");
    let sql_string = sql_string.replace("$table_fields", &User::sql_table_fields());
    let sql_string = client.prepare(&sql_string).await.unwrap();

    let results = client
        .query(&sql_string, &[])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();
    Ok(results)
}

pub async fn add_user(client: &Client, user_info: MiniUser) -> Result<User, MyError> {
    let _stmt = include_str!("../sql/add_user.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &user_info.steamid,
                &user_info.username,
                &user_info.avatarurl,
            ],
        )
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound) // more applicable for SELECTs
}
