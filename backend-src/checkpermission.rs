use serde::Serialize;

use crate::models::User;

#[derive(Clone, Copy)]
pub enum UserPermission {
    None = 0,
    /// All permissions.
    Admin = 1 << 0,
    /// Can set permissions for others.
    SetPermissions = 1 << 1,
    /// Can create a new League, and modify existing ones.
    CreateLeague = 1 << 2,
    /// Can create a new Game between two teams for an existing League.
    CreateGame = 1 << 3,
}

impl Serialize for UserPermission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(*self as i64)
    }
}

impl User {
    /// Perform bitwise AND operation on the permission bitfield to see if it contains `permission`.
    fn check_has_permission(&self, permission: UserPermission) -> bool {
        self.permissions & (permission as i64) != 0
    }
}

/// # Premade Permissions
/// Some example premade permission shorthands in order to check multiple permissions at once,
/// or to quickly set a user's permission without specifying each line manually.
mod premade_permissions {
    use crate::checkpermission::UserPermission;
    pub static LEAGUE_ADMIN: i64 =
        (UserPermission::CreateLeague as i64) + (UserPermission::CreateGame as i64);
}
