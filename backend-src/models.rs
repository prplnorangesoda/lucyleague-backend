use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")]
pub struct MiniUser {
    pub steamid: String,
    pub username: String,
}

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub id: i64,
    pub steamid: String,
    pub username: String,
}

impl From<User> for MiniUser {
    fn from(value: User) -> Self {
        MiniUser {
            steamid: value.steamid,
            username: value.username,
        }
    }
}
#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "teams")]
pub struct Team {
    pub id: i64,
    pub team_name: String,
}

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "userTeam")]
pub struct UserTeam {
    pub userid: String,
    pub teamid: String,
}

#[derive(Deserialize, PostgresMapper, Serialize, Debug)]
#[pg_mapper(table = "authorization")]
pub struct Authorization {
    pub userid: i64,
    pub token: String,
    pub expires: DateTime<chrono::Utc>,
}
