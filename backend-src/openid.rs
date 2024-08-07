// Definition file for Steam's OpenID 2.0.
use urlencoding::encode;

use crate::config::ExampleConfig;
pub enum OpenIdMode {
    //Immediate,
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
&openid.return_to=https%3A%2F%2Frgl.gg%2FLogin%2FDefault.aspx%3FReturnUrl%3Dhttps%253A%252F%252Frgl.gg%252FPublic%252Fabout%252FFrontPage%252FIntro%252Fdefault.aspx%26push%3D1%26url%3D~%252FPublic%252FAbout%252FFrontPage%252FIntro%252FDefault.aspx%26dnoa.userSuppliedIdentifier%3Dhttps%253A%252F%252Fsteamcommunity.com%252Fopenid%252F
&openid.realm=https%3A%2F%2Frgl.gg%2F
&openid.mode=checkid_setup
&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
*/
/*
https://rgl.gg/Login/Default.aspx?push=1&r=40
&dnoa.userSuppliedIdentifier=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2F
&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0
&openid.mode=id_res
&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin
&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198025040446
&openid.return_to=https%3A%2F%2Frgl.gg%2FLogin%2FDefault.aspx%3Fpush%3D1%26r%3D40%26dnoa.userSuppliedIdentifier%3Dhttps%253A%252F%252Fsteamcommunity.com%252Fopenid%252F
&openid.response_nonce=2024-07-27T16%3A07%3A06Zdg9%2BzW7ALLLycjtF7T7mWe3qKp0%3D
&openid.assoc_handle=34321234
&openid.signed=
signed%2C
op_endpoint%2C
claimed_id%2C
identity%2C
return_to%2C
response_nonce%2C
assoc_handle
&openid.sig=f9dFKCcwpaGUWp2VsXwMV7csgsU%3D */

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
            &self.server_config.openid_realm, &self.server_config.server_port
        );
        let mode = format!(
            "openid.mode={0}",
            match &self.config.mode {
                //OpenIdMode::Immediate => "checkid_immediate",
                OpenIdMode::Setup => "checkid_setup",
            }
        );
        let ns = "openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0";

        format!("{root_part}?{claimed_id}&{identity}&{return_part}&{realm}&{mode}&{ns}")
    }
}
