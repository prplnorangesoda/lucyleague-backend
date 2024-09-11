// Initial configuration for the server.
use confik::Configuration;
use serde::Deserialize;

#[derive(Debug, Default, Configuration, Clone)]
pub struct ExampleConfig {
    pub root_user_steamid: Option<String>,
    pub steam_api_key: String,
    pub openid_realm: String,
    pub openid_port: String,
    pub server_addr: String,
    pub server_port: u16,
    #[confik(from = DbConfig)]
    pub pg: deadpool_postgres::Config,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct DbConfig(deadpool_postgres::Config);

impl From<DbConfig> for deadpool_postgres::Config {
    fn from(value: DbConfig) -> Self {
        value.0
    }
}

impl confik::Configuration for DbConfig {
    type Builder = Option<Self>;
}
