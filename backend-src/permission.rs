//! # About permissions
//! Permissions are made using a bitfield.
//! Using a i64 allows us 63 different permissions with one numerical value.
//! We use an i64 to be able to store this permissions value in Postgres.
//! This should be enough for now, but in future we can use a second permission value.
use std::ops::BitAnd;

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

impl BitAnd for UserPermission {
    type Output = i64;
    fn bitand(self, rhs: Self) -> Self::Output {
        self as i64 & rhs as i64
    }
}

impl BitAnd<i64> for UserPermission {
    type Output = i64;
    fn bitand(self, rhs: i64) -> Self::Output {
        self as i64 & rhs
    }
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
    /// Check if the user has _either_ the permission or admin
    pub fn admin_or_perm(&self, permission: UserPermission) -> bool {
        (self.check_has_permission(UserPermission::Admin))
            || (self.check_has_permission(permission))
    }
    /// Perform bitwise AND operation on the permission bitfield to see if it contains `permission`.
    pub fn check_has_permission(&self, permission: UserPermission) -> bool {
        permission & self.permissions != 0
    }
    pub fn add_permission(&mut self, permission: UserPermission) {
        if self.check_has_permission(permission) {
            return;
        }
        self.permissions += permission as i64;
    }
}

/// # Premade Permissions
/// Some example premade permission shorthands in order to check multiple permissions at once,
/// or to quickly set a user's permission without specifying each line manually.
pub mod premade_permissions {
    use crate::permission::UserPermission;
    pub static LEAGUE_ADMIN: i64 =
        (UserPermission::CreateLeague as i64) + (UserPermission::CreateGame as i64);
}
