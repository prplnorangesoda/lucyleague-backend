use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{errors::MyError, models::User};

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

pub async fn add_user(client: &Client, user_info: User) -> Result<User, MyError> {
    let _stmt = include_str!("../sql/add_user.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &user_info.email,
                &user_info.first_name,
                &user_info.last_name,
                &user_info.username,
            ],
        )
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound) // more applicable for SELECTs
}
