use urlencoding::encode;

use crate::config::ExampleConfig;
pub enum OpenIdMode {
    Immediate,
    Setup,
}

pub struct SteamOpenIdConfig {
    return_url: String,
    mode: OpenIdMode,
    identity: &'static str,
}

impl SteamOpenIdConfig {
    pub fn new(return_url: &str) -> Self {
        SteamOpenIdConfig {
            return_url: String::from(return_url),
            mode: OpenIdMode::Setup,
            identity: "http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select",
        }
    }
}

pub struct SteamOpenId {
    config: SteamOpenIdConfig,
    server_config: ExampleConfig,
}

/*
https://steamcommunity.com/openid/login
?openid.claimed_id=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select
&openid.identity=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select
&openid.return_to=https%3A%2F%2Fexample.com%26dnoa.userSuppliedIdentifier%3Dhttps%3A%2F%2Fsteamcommunity.com%2Fopenid%2F
&openid.realm=https%3A%2F%2Fexample.com%2F
&openid.mode=checkid_setup
&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0

*/
impl SteamOpenId {
    pub fn new(config: SteamOpenIdConfig, server_config: ExampleConfig) -> SteamOpenId {
        SteamOpenId {
            config,
            server_config,
        }
    }
    pub fn get_auth_url(&self) -> String {
        let root_part = "https://steamcommunity.com/openid/login";
        let claimed_id =
            "openid.claimed_id=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select";
        let identity = format!("&openid.identity={0}", &self.config.identity);
        let return_part = format!("openid.return_to={0}", encode(&self.config.return_url));
        let realm = format!(
            "openid.realm=http%3A%2F%2F{0}:{1}%2F",
            &self.server_config.server_addr, &self.server_config.server_port
        );
        let mode = format!(
            "openid.mode={0}",
            match &self.config.mode {
                OpenIdMode::Immediate => "checkid_immediate",
                OpenIdMode::Setup => "checkid_setup",
            }
        );
        let ns = "openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0";

        format!("{root_part}?{claimed_id}&{identity}&{return_part}&{realm}&{mode}&{ns}")
    }
}
