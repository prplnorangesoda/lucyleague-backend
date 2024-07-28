use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    errors::MyError,
    models::{Authorization, MiniUser, User},
};

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

pub async fn get_authorization_from_user(client: &Client, User: &User) {}

pub async fn register_authorization(
    client: &Client,
    token: &str,
    user: &User,
) -> Result<Authorization, MyError> {
    let _stmt = include_str!("../sql/register_auth_token.sql");
    let _stmt = _stmt.replace("$fields", &Authorization::sql_fields());
    println!("{}", &_stmt);
    println!("{}, {}", &user.id, &token);
    let stmt = client.prepare(&_stmt).await?;

    client
        .query(&stmt, &[&user.id, &token])
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
        .query(&stmt, &[&user_info.steamid, &user_info.username])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound) // more applicable for SELECTs
}
