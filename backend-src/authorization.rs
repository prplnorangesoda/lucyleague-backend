// Code relating to the Authorization model - generation, etc.
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Months};
use deadpool_postgres::Client;
use randomizer::Randomizer;

use crate::db::register_authorization;
use crate::errors::MyError;
use crate::models::{Authorization, User};

pub async fn get_authorization_for_user(
    dbclient: &Client,
    user: &User,
) -> Result<Authorization, MyError> {
    crate::db::get_authorization_for_user(dbclient, user).await
}

pub async fn create_authorization_for_user(
    dbclient: &Client,
    user: &User,
) -> Result<Authorization, MyError> {
    let token = Randomizer::ALPHANUMERIC(40).string().unwrap();
    let now = SystemTime::now();
    let time: i64 = now
        .duration_since(UNIX_EPOCH)
        .expect("Somehow before unix epoch")
        .as_secs()
        .try_into()
        .unwrap();

    let date_time: DateTime<chrono::Utc> =
        DateTime::from_timestamp(time, 0).expect("Timestamp invalid");

    register_authorization(
        dbclient,
        &token,
        user,
        date_time
            .checked_add_months(Months::new(3))
            .expect("error adding time to expiry"),
    )
    .await
}
