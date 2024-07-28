use deadpool_postgres::Client;
use randomizer::{Charset, Randomizer};

use crate::db::register_authorization;
use crate::errors::MyError;
use crate::models::{Authorization, User};

pub async fn get_authorization_for_user(
    dbclient: &Client,
    user: &User,
) -> Result<Authorization, MyError> {
    create_authorization_for_user(&dbclient, &user).await
}

pub async fn create_authorization_for_user(
    dbclient: &Client,
    user: &User,
) -> Result<Authorization, MyError> {
    let token = "supersecure".to_string();
    register_authorization(dbclient, &token, user).await
}
