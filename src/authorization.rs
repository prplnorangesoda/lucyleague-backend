// Code relating to the Authorization model - generation, etc.
use chrono::Months;
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

    let time_now = chrono::offset::Utc::now();

    register_authorization(
        dbclient,
        &token,
        user,
        time_now
            .checked_add_months(Months::new(1))
            .expect("error adding time to expiry"),
    )
    .await
}
