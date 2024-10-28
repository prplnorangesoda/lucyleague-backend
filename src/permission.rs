//! # About permissions
//! Permissions are made using a bitfield.
//! Using a i64 allows us 63 different permissions with one numerical value.
//! We use an i64 to be able to store this permissions value in Postgres.
//! This should be enough for now, but in future we can use a second permission value.
use std::ops::BitAnd;

use bitflags::bitflags;
use num_derive::FromPrimitive;
use serde::Serialize;

use crate::models::User;
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct UserPermission: i64 {
        /// Must check for none using ==.
        const None = 0;
        /// All permissions.
        const Admin = 1 << 0;
        /// Can set permissions for others.
        const SetPermissions = 1 << 1;
        /// Can create a new League, and modify existing ones.
        const CreateLeague = 1 << 2;
        /// Can create a new Game between two teams for an existing League.
        const CreateGame = 1 << 3;
        /// Can create new Teams and manage existing ones.
        const CreateTeam = 1 << 4;
        // # Premade Permissions
        // Some example premade permission shorthands in order to check multiple permissions at once,
        // or to quickly set a user's permission without specifying each line manually.

        /// Permissions to create leagues and games
        const LeagueAdmin = Self::CreateLeague.bits() | Self::CreateGame.bits();
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
        permission.bits() & self.permissions != 0
    }
    pub fn add_permission(&mut self, permission: UserPermission) {
        if self.check_has_permission(permission) {
            return;
        }
        self.permissions += permission.bits();
    }
}
