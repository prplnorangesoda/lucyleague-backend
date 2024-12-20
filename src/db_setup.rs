use std::{io::Read, path::PathBuf};

use deadpool_postgres::Client;
use derive_more::{Display, Error, From};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::models;

async fn destroy(client: &Client) -> Result<u64, tokio_postgres::Error> {
    client
        .execute("DROP SCHEMA IF EXISTS ll CASCADE;", &[])
        .await
}
pub fn init(client: &Client) {}

#[derive(Debug, Display, Error, From)]
pub enum MigrationError {
    InvalidDb,
    Io(std::io::Error),
    PGError(tokio_postgres::Error),
}

static MIGRATION_DIR: &str = "./sql/migrations";

use MigrationError::*;
///
///
pub async fn migrate_from(
    client: &Client,
    version: i32,
    allow_schema_destruction: bool,
) -> Result<(), MigrationError> {
    if version < 0 {
        if !allow_schema_destruction {
            return Err(InvalidDb);
        }
        destroy(client).await.map_err(MigrationError::PGError)?;
    }

    use std::fs;
    let migration_count = fs::read_dir(MIGRATION_DIR).unwrap().count();
    let migrations = fs::read_dir(MIGRATION_DIR).unwrap();
    let migration_idx: Vec<(i32, PathBuf)> = Vec::with_capacity(migration_count);
    for entry in migrations {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_contents = fs::read(&path)?;
        let migration = String::from_utf8_lossy(&file_contents);

        log::debug!("Executing migration {0}: {migration}", path.display());
    }
    Ok(())
}
pub async fn version(client: &Client) -> Option<i32> {
    let version_exists: bool = client
        .query_one(
            "SELECT EXISTS (SELECT FROM pg_tables WHERE tablename = 'config' AND schemaname = 'll');",
            &[],
        )
        .await
        .unwrap()
        .get(0);

    if !version_exists {
        return None;
    }

    let stmt = "SELECT $table_fields FROM config LIMIT 1;"
        .replace("$table_fields", &models::Config::sql_table_fields());
    let version = client
        .query_one(&stmt, &[])
        .await
        .expect("version data should not become unavailable before function ends")
        .get("version");

    Some(version)
}
