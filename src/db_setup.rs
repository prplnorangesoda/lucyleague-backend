use std::{env::current_dir, io::Read, path::PathBuf};

use deadpool_postgres::Client;
use derive_more::{Display, Error, From};
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::models;

async fn destroy(client: &Client) -> Result<u64, tokio_postgres::Error> {
    client
        .execute("DROP SCHEMA IF EXISTS ll CASCADE;", &[])
        .await
}

#[derive(Debug, Display, Error, From)]
pub enum MigrationError {
    InvalidDb,
    Io(std::io::Error),
    PGError(tokio_postgres::Error),
}

static MIGRATION_DIR: &str = "./sql/migrations";

use MigrationError::*;

/// Migrate from a specified database version.
///
/// If 'version' is < 0, errors, unless
/// allow_schema_destruction is true.
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
    log::debug!("MIGRATION_DIR: {MIGRATION_DIR}");
    log::debug!("CURRENT DIR: {}", current_dir().unwrap().display());
    let migration_count = fs::read_dir(MIGRATION_DIR).unwrap().count();
    let migrations = fs::read_dir(MIGRATION_DIR).unwrap();
    let mut migration_idx: Vec<(i32, PathBuf)> = Vec::with_capacity(migration_count);
    for entry in migrations {
        let entry = entry.unwrap();
        let path = entry.path();
        log::trace!("MIGRATION PATH: {}", path.display());

        let index: i32 = match atoi::atoi(entry.file_name().as_os_str().as_encoded_bytes()) {
            Some(i) => i,
            None => continue,
        };

        migration_idx.push((index, path));
    }
    migration_idx.sort_by_key(|val| val.0);
    for migration in migration_idx {
        if migration.0 <= version {
            continue;
        }
        let file_contents = fs::read(&migration.1)?;
        let code = String::from_utf8_lossy(&file_contents);

        log::debug!("Executing migration {0}", migration.0,);
        log::trace!("Migration {0}: {1}", migration.0, migration.1.display());
        client
            .batch_execute(&code)
            .await
            .expect("migration execution should not fail");
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

    let stmt = "SELECT $table_fields FROM ll.config LIMIT 1;".replacen(
        "$table_fields",
        &models::Config::sql_table_fields(),
        1,
    );
    let version = client
        .query_one(&stmt, &[])
        .await
        .expect("version data should not become unavailable before function ends")
        .get("version");

    Some(version)
}
