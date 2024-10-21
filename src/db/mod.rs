// Code that acts as a wrapper for database values.
use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::{
    authorization::create_authorization_for_user,
    errors::MyError,
    models::{
        Authorization, League, MiniLeague, MiniTeam, MiniUser, Team, TeamDivAssociation, User,
        UserTeam,
    },
    permission::UserPermission,
};

pub mod divisions;
pub mod leagues;
pub mod teams;

pub async fn add_test_data(client: &Client) -> Result<(), MyError> {
    let _stmt = include_str!("../../sql/test_data.sql");

    client.batch_execute(_stmt).await?;
    Ok(())
}
pub async fn initdb(client: &Client) -> Result<(), MyError> {
    let _stmt = include_str!("../../sql/initdb.sql");

    client.batch_execute(_stmt).await?;
    Ok(())
}

pub async fn revoke_user_authorization(client: &Client, user: &User) -> Result<u64, MyError> {
    let stmt = client
        .prepare("DELETE FROM authorizations WHERE userid=$1;")
        .await
        .unwrap();

    client
        .execute(&stmt, &[&user.id])
        .await
        .map_err(MyError::PGError)
}

pub async fn get_team_from_id(client: &Client, team_id: i64) -> Result<Team, MyError> {
    let _stmt = "SELECT $table_fields FROM teams WHERE id=$1";
    let _stmt = _stmt.replace("$table_fields", &Team::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&team_id])
        .await?
        .iter()
        .map(|row| Team::from_row_ref(row).unwrap())
        .collect::<Vec<Team>>()
        .pop()
        .ok_or(MyError::NotFound);

    results
}

pub async fn get_teamdivassociation_from_id(
    client: &Client,
    assoc_id: i64,
) -> Result<TeamDivAssociation, MyError> {
    let _stmt = "SELECT $table_fields FROM teamDivAssociations WHERE id=$1";
    let _stmt = _stmt.replace("$table_fields", &Team::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let results = client
        .query(&stmt, &[&assoc_id])
        .await?
        .iter()
        .map(|row| TeamDivAssociation::from_row_ref(row).unwrap())
        .collect::<Vec<TeamDivAssociation>>()
        .pop()
        .ok_or(MyError::NotFound);

    results
}

pub async fn add_team(client: &Client, team: &MiniTeam) -> Result<Team, MyError> {
    let _stmt = "INSERT INTO teams(team_tag, team_name, created_at, owner_id)
    VALUES
    ($1, $2, $3, $4)
    RETURNING 
    $table_fields";
    let _stmt = _stmt.replace("$table_fields", &Team::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let time_now = chrono::offset::Utc::now();

    client
        .query(
            &stmt,
            &[&team.team_tag, &team.team_name, &time_now, &team.owner_id],
        )
        .await?
        .iter()
        .map(|row| Team::from_row_ref(row).unwrap())
        .collect::<Vec<Team>>()
        .pop()
        .ok_or(MyError::NotFound)
}

pub async fn get_team_players(
    client: &Client,
    team: &TeamDivAssociation,
) -> Result<Vec<User>, MyError> {
    let _stmt = "SELECT $table_fields FROM userTeam WHERE teamid=$1 AND divisionid=$2";
    let _stmt = _stmt.replace("$table_fields", &UserTeam::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let userids: Vec<i64> = client
        .query(&stmt, &[&team.id, &team.divisionid])
        .await?
        .iter()
        .map(|row| UserTeam::from_row_ref(row).unwrap().userid)
        .collect();

    mass_get_user_from_internal_id(client, &userids).await
}

pub async fn get_user_from_internal_id(client: &Client, userid: i64) -> Result<User, MyError> {
    let _stmt = "SELECT $table_fields FROM users WHERE id=$1";
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let row = client
        .query_one(&stmt, &[&userid])
        .await
        .map_err(|_| MyError::NotFound)?;
    Ok(User::from_row_ref(&row).unwrap())
}

pub async fn mass_get_user_from_internal_id(
    client: &Client,
    userids: &Vec<i64>,
) -> Result<Vec<User>, MyError> {
    let _stmt = "SELECT $table_fields FROM users WHERE id IN ($1)";
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let users: Vec<User> = client
        .query(&stmt, &[userids])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect();

    Ok(users)
}

// pub async fn add_team(client: &Client, league: League, team: &MiniTeam) -> Result<Team, MyError> {
//     let _stmt = "INSERT INTO teams(leagueid, teamname)"
// }

pub async fn add_league(client: &Client, league: MiniLeague) -> Result<League, MyError> {
    let _stmt = "INSERT INTO leagues(name, accepting_teams, is_hidden, created_at) VALUES ($1, $2, $3, $4) RETURNING $table_fields";
    let _stmt = _stmt.replace("$table_fields", &League::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let time_now = chrono::offset::Utc::now();

    let results = client
        .query(
            &stmt,
            &[
                &league.name,
                &league.accepting_teams,
                &league.is_hidden,
                &time_now,
            ],
        )
        .await?
        .iter()
        .map(|row| League::from_row_ref(row).unwrap())
        .collect::<Vec<League>>()
        .pop()
        .unwrap();
    Ok(results)
}
pub async fn get_user_from_auth_token(client: &Client, token: &str) -> Result<User, MyError> {
    let _stmt = include_str!("../../sql/get_user_from_authtoken.sql");
    let stmt = client.prepare(_stmt).await.unwrap();

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

    client
        .query(&stmt, &[&received_auth.userid])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound)
}
pub async fn get_user_from_steamid(client: &Client, steamid: &str) -> Result<User, MyError> {
    let _stmt = include_str!("../../sql/get_user_from_steamid.sql");
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
    let _stmt = include_str!("../../sql/get_auth_token.sql");
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
    let _stmt = include_str!("../../sql/register_auth_token.sql");
    // $table_fields didn't work with this for some reason when i tested it
    let _stmt = _stmt.replace("$fields", &Authorization::sql_fields());
    log::debug!("Registering authorization {token} for {0}", &user.id);
    let stmt = client.prepare(&_stmt).await?;
    let time_now = chrono::offset::Utc::now();

    client
        .query(&stmt, &[&user.id, &token, &time_now, &expiry])
        .await?
        .iter()
        .map(|row| Authorization::from_row_ref(row).unwrap())
        .collect::<Vec<Authorization>>()
        .pop()
        .ok_or(MyError::NotFound)
}

pub async fn get_users(client: &Client) -> Result<Vec<User>, MyError> {
    let sql_string = include_str!("../../sql/get_users.sql");
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

pub async fn set_user_permissions(
    client: &Client,
    user: &User,
    permissions: i64,
) -> Result<User, MyError> {
    let _stmt = "UPDATE users SET permissions=$1 WHERE id=$2 RETURNING $table_fields";
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let resp = client
        .query(&stmt, &[&permissions, &user.id])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound);

    resp
}

pub async fn set_super_user(client: &Client, user: &User) -> Result<User, MyError> {
    set_user_permissions(client, user, UserPermission::Admin as i64).await
}

pub async fn add_user(client: &Client, user_info: MiniUser) -> Result<User, MyError> {
    let _stmt = include_str!("../../sql/add_user.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let date = chrono::offset::Utc::now();

    let resp = client
        .query(
            &stmt,
            &[
                &user_info.steamid,
                &user_info.username,
                &user_info.avatarurl,
                &date,
            ],
        )
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound); // more applicable for SELECTs
    resp
}

pub async fn get_user_count(client: &Client) -> Result<i64, MyError> {
    let stmt = client.prepare("SELECT COUNT(*) FROM users").await.unwrap();

    let resp: i64 = client.query_one(&stmt, &[]).await?.get(0);

    Ok(resp)
}

pub async fn get_user_page(
    client: &Client,
    page: u32,
    amount: std::num::NonZero<u32>,
) -> Result<Vec<User>, MyError> {
    log::trace!("Getting page {page} amount {amount}");
    let _stmt = include_str!("../../sql/get_users_paged.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let amount: u32 = amount.into();
    let amount: i64 = amount.into();
    let page: i64 = page.into();
    let offset: i64 = page * amount;

    let resp = client
        .query(&stmt, &[&offset, &amount])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect();
    Ok(resp)
}

pub async fn search_usernames(
    client: &Client,
    search_term: &str,
    page: u32,
    amount: std::num::NonZero<u32>,
) -> Result<Vec<User>, MyError> {
    log::trace!("Searching DB with term {search_term}");

    let amount: u32 = amount.into();
    let amount: i64 = amount.into();
    let page: i64 = page.into();
    let offset: i64 = page * amount;

    let _stmt = include_str!("../../sql/fuzzy_search.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    let resp = client
        .query(&stmt, &[&search_term, &amount, &offset])
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect();
    Ok(resp)
}
