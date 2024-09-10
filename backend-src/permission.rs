//! # About permissions
//! Permissions are made using a bitfield.
//! Using a i64 allows us 63 different permissions with one numerical value.
//! We use an i64 to be able to store this permissions value in Postgres.
//! This should be enough for now, but in future we can use a second permission value.
use num_derive::FromPrimitive;
use serde::Serialize;

use crate::models::User;

#[derive(Clone, Copy, FromPrimitive)]
pub enum UserPermission {
    /// Must check for none using ==.
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
        serializer.serialize_u64(*self as u64)
    }
}

impl User {
    /// Perform bitwise AND operation on the permission bitfield to see if it contains `permission`.
    fn check_has_permission(&self, permission: UserPermission) -> bool {
        self.permissions & (permission as i64) != 0
    }
    fn add_permission(&mut self, permission: UserPermission) {
        if self.permissions & (permission as i64) != 0 {
            return
        }
        self.permissions = self.permissions + permission as i64;
    }
}

/// # Premade Permissions
/// Some example premade permission shorthands in order to check multiple permissions at once,
/// or to quickly set a user's permission without specifying each line manually.
pub mod premade_permissions {
<<<<<<< HEAD:backend-src/permission.rs
    use crate::permission::UserPermission;
=======
    use crate::checkpermission::UserPermission;
    pub static ALL: i64 = UserPermission::Admin as i64;
>>>>>>> 3e4f4cd1bdc0d8b3537b8d1e1b7910eb4cdeb023:backend-src/checkpermission.rs
    pub static LEAGUE_ADMIN: i64 =
        (UserPermission::CreateLeague as i64) + (UserPermission::CreateGame as i64);
}
