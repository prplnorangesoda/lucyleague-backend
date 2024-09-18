use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

/// A user without the id / created_at.
///
/// Useful if you want Postgres to generate an id automatically.
#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")]
pub struct MiniUser {
    pub steamid: String,
    pub permissions: Option<i64>,
    pub avatarurl: String,
    pub username: String,
}

/// A basic user / player.
///
///
#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    /// Postgres' Primary Key, for all intents and purposes use `steamid`.
    pub id: i64,
    /// A bitfield of `crate::checkpermission::UserPermission`.
    pub permissions: i64,
    pub avatarurl: String,
    pub steamid: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for MiniUser {
    fn from(value: User) -> Self {
        MiniUser {
            steamid: value.steamid,
            username: value.username,
            avatarurl: value.avatarurl,
            permissions: Some(value.permissions),
        }
    }
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "leagues")]
pub struct MiniLeague {
    pub name: String,
    pub accepting_teams: bool,
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "leagues")]
pub struct League {
    pub id: i64,
    pub name: String,
    pub accepting_teams: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "teams")]
pub struct MiniTeam {
    pub leagueid: i64,
    pub team_name: String,
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "teams")]
pub struct Team {
    pub id: i64,
    pub leagueid: i64,
    pub team_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "userTeam")]
pub struct UserTeam {
    pub leagueid: i64,
    pub userid: i64,
    pub teamid: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "authorization")]
pub struct Authorization {
    pub userid: i64,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires: DateTime<Utc>,
}