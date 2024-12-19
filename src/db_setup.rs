use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::Error;

use crate::models;

async fn destroy(client: &Client) -> Result<u64, Error> {
    client
        .execute("DROP SCHEMA IF EXISTS ll CASCADE;", &[])
        .await
}
pub fn init(client: &Client) {}

///
///
pub async fn migrate_from(
    client: &Client,
    version: i32,
    allow_schema_destruction: bool,
) -> Result<(), &'static str> {
    if version < 0 {
        if !allow_schema_destruction {
            return Err("Version is invalid / OOB and destruction was disallowed");
        }
        destroy(client);
    }
    Ok(())
}
pub async fn version(client: &Client) -> Option<i32> {
    let version_exists = client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'config' AND schemaname = 'll');",
            &[],
        )
        .await
        .is_ok();
    if !version_exists {
        return None;
    }

    let stmt = "SELECT $table_fields FROM config LIMIT 1;"
        .replace("$table_fields", &models::Config::sql_table_fields());
    let version = client.query_one(&stmt, &[]).await.unwrap().get("version");

    Some(version)
}
