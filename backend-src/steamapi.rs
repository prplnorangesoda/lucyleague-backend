use std::collections::HashMap;

// Code that interacts with Steam's Web API.
use derive_more::{Display, Error, From};

/// An error caused by our interacting with the steam API.
#[derive(Debug, Display, Error, From)]
pub enum ApiError {
    Reqwest(reqwest::Error),
    Handling,
    NotFound,
    KeyValues(steamopenid::kv::DecodeError),
    Serde(serde_json::Error),
}
/// The level of access we have to the user's profile, and the according data.
pub enum ReturnedAccessLevel {
    /// The profile is visible to us.
    All = 3,
    /// The profile is not visible to us.
    Private = 1,
}

#[derive(serde::Deserialize, serde::Serialize, From)]
pub enum PlayerSummaryAccess {
    All {
        private: Box<PrivatelyAvailableSummary>,
        public: Box<PubliclyAvailableSummary>,
    },
    Private {
        public: Box<PubliclyAvailableSummary>,
    },
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PubliclyAvailableSummary {
    /// 64bit SteamID of the user.
    pub steamid: String,
    /// The user's display name.
    pub personaname: String,
    /// The full URL of the player's Steam Community profile.
    pub profileurl: String,
    /// The full URL of the player's 32x32 avatar.
    pub avatar: String,
    /// The full URL of the player's 64x64 avatar.
    pub avatarmedium: String,
    /// The full URL of the player's 184x184 avatar.
    pub avatarfull: String,
    /// Current user status. If the profile is private, will always be Offline.
    pub personastate: i64,
    /// Is the profile set up.
    pub profilestate: i64,
    /// The last time the user was online, in unix time.
    pub lastlogoff: i64,
    /// Are comments allowed?
    pub commentpermission: i64,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PrivatelyAvailableSummary {
    pub realname: String,
    pub primaryclanid: String,
    pub timecreated: i64,
    pub gameid: Option<String>,
    pub gameserverip: Option<String>,
    pub gameextrainfo: Option<String>,
    #[deprecated(note = "use loccityid")]
    pub cityid: Option<String>,
    pub loccountrycode: Option<String>,
    pub locstatecode: Option<String>,
    pub loccityid: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SteamResponseAllInfo {
    response: SteamResponseAllInfoInner,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SteamResponseAllInfoInner {
    players: Vec<AllPlayerInfo>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SteamResponsePublicInfo {
    response: SteamResponsePublicInfoInner,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SteamResponsePublicInfoInner {
    players: Vec<PubliclyAvailableSummary>,
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SteamReturnInfo {
    response: SteamReturnInfoInner,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct SteamReturnInfoInner {
    players: Vec<BasicInfoIHateWritingThis>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct BasicInfoIHateWritingThis {
    communityvisibilitystate: i32,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct AllPlayerInfo {
    pub realname: String,
    pub primaryclanid: String,
    pub timecreated: i64,
    pub gameid: Option<String>,
    pub gameserverip: Option<String>,
    pub gameextrainfo: Option<String>,
    #[deprecated(note = "use loccityid")]
    pub cityid: Option<String>,
    pub loccountrycode: Option<String>,
    pub locstatecode: Option<String>,
    pub loccityid: Option<String>,
    /// 64bit SteamID of the user.
    pub steamid: String,
    /// The user's display name.
    pub personaname: String,
    /// The full URL of the player's Steam Community profile.
    pub profileurl: String,
    /// The full URL of the player's 32x32 avatar.
    pub avatar: String,
    /// The full URL of the player's 64x64 avatar.
    pub avatarmedium: String,
    /// The full URL of the player's 184x184 avatar.
    pub avatarfull: String,
    /// Current user status. If the profile is private, will always be Offline.
    pub personastate: i64,
    /// Is the profile set up.
    pub profilestate: i64,
    /// The last time the user was online, in unix time.
    pub lastlogoff: i64,
    /// Are comments allowed?
    pub commentpermission: i64,
}

impl PubliclyAvailableSummary {
    fn from_allplayerinfo(value: &AllPlayerInfo) -> Self {
        let serialised = serde_json::to_string(&value).unwrap();
        serde_json::from_str(&serialised).unwrap()
    }
}

impl PrivatelyAvailableSummary {
    fn from_allplayerinfo(value: &AllPlayerInfo) -> Self {
        let serialised = serde_json::to_string(&value).unwrap();
        serde_json::from_str(&serialised).unwrap()
    }
}

pub async fn get_user_summary(
    steam_api_key: &String,
    steamid: &str,
) -> Result<PlayerSummaryAccess, ApiError> {
    log::trace!("Checking steam summary for player with id {steamid}");
    let url = format!(
        "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={0}&steamids={1}",
        steam_api_key, steamid
    );
    let body = reqwest::get(url).await?.text().await?;
    log::debug!("Returned info from Steam: {}", body);

    let mut basic_info = serde_json::from_str::<SteamReturnInfo>(&body)?;

    log::debug!("Steam basic return info: {basic_info:?}");
    let response_type = match basic_info
        .response
        .players
        .pop()
        .ok_or(ApiError::Handling)?
        .communityvisibilitystate
    {
        3 => ReturnedAccessLevel::All,
        1 => ReturnedAccessLevel::Private,
        _ => {
            return Err(ApiError::Handling);
        }
    };

    // We can see the whole profile, therefore the response includes everything
    if let ReturnedAccessLevel::All = response_type {
        log::debug!("Steam response matched All ReturnedAccessLevel");
        let needed_info = serde_json::from_str::<SteamResponseAllInfo>(&body)?
            .response
            .players
            .pop()
            .ok_or(ApiError::NotFound)?;
        Ok(PlayerSummaryAccess::All {
            private: Box::new(PrivatelyAvailableSummary::from_allplayerinfo(&needed_info)),
            public: Box::new(PubliclyAvailableSummary::from_allplayerinfo(&needed_info)),
        })
    }
    // We can't see the whole profile, therefore the response includes only public information.
    else {
        log::debug!("Steam response matched Private ReturnedAccessLevel");
        Ok(PlayerSummaryAccess::Private {
            public: Box::new(serde_json::from_str(&body)?),
        })
    }
}

impl From<steamopenid::ApiError> for ApiError {
    fn from(value: steamopenid::ApiError) -> Self {
        match value {
            steamopenid::ApiError::Handling => Self::Handling,
            steamopenid::ApiError::KeyValuesError(h) => Self::KeyValues(h),
        }
    }
}

pub async fn verify_authentication_with_steam(
    key_values_map: &HashMap<String, String>,
) -> Result<bool, ApiError> {
    steamopenid::verify_auth_keyvalues(key_values_map)
        .await
        .map_err(|err| err.into())
}

pub async fn verify_auth_underscores(key_values_map: &str) -> Result<bool, ApiError> {
    let map: HashMap<String, String> =
        serde_json::from_str(&key_values_map.replace("__", ".")).unwrap();
    verify_authentication_with_steam(&map).await
}
