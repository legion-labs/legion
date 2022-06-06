use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::Hash,
    str::FromStr,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Error, Result};

#[macro_export]
macro_rules! declare_built_in_roles {
    ($($name:ident => $description:literal), *$(,)?) => {
        impl crate::types::RoleId {
            $(
            pub const $name: Self =
                crate::types::RoleId(std::borrow::Cow::Borrowed(casey::lower!(stringify!($name))));
            )*

            const ALL_BY_NAME: &'static [(&'static str, &'static Self)] = &[
                $(
                    (casey::lower!(stringify!($name)), &Self::$name),
                )*
            ];

            pub(crate) fn get_built_in(s: &str) -> Option<&'static Self> {
                Self::ALL_BY_NAME.iter().find(|(name, _)| *name == s).map(|(_, id)| *id)
            }

            pub fn is_built_in(&self) -> bool {
                Self::ALL_BY_NAME.iter().any(|(name, _)| *name == self.0.as_ref())
            }

            pub const BUILT_INS: &'static [&'static Self] = &[
                $(
                    &Self::$name,
                )*
            ];
        }

        impl crate::types::Role {
            $(
            pub const $name: Self = Self {
                id: crate::types::RoleId::$name,
                description: std::borrow::Cow::Borrowed($description),
                created_at: chrono::MIN_DATETIME,
            };
            )*

            const ALL_BY_ID: &'static [(&'static crate::types::RoleId, &'static Self)] = &[
                $(
                    (&crate::types::RoleId::$name, &Self::$name),
                )*
            ];

            pub(crate) fn get_built_in(role_id: &crate::types::RoleId) -> Option<&'static Self> {
                Self::ALL_BY_ID.iter().find(|(id, _)| *id == role_id).map(|(_, role)| *role)
            }

            pub fn is_built_in(&self) -> bool {
                Self::get_built_in(&self.id).is_some()
            }

            pub const BUILT_INS: &'static [&'static crate::types::Role] = &[
                $(
                    &Self::$name,
                )*
            ];
        }
    };
}

/// A role identifier.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
pub struct RoleId(pub(crate) Cow<'static, str>);

impl Display for RoleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RoleId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidRoleId(s.to_string()));
        }

        if s.contains(|c: char| !c.is_ascii_alphanumeric()) {
            return Err(Error::InvalidRoleId(s.to_string()));
        }

        Ok(if let Some(built_in) = RoleId::get_built_in(s) {
            built_in.clone()
        } else {
            RoleId(Cow::Owned(s.to_string()))
        })
    }
}

impl<'a> From<RoleId> for crate::api::common::RoleId {
    fn from(role_id: RoleId) -> Self {
        Self(role_id.0.to_string())
    }
}

impl<'a> TryFrom<crate::api::common::RoleId> for RoleId {
    type Error = Error;

    fn try_from(role_id: crate::api::common::RoleId) -> Result<Self> {
        role_id.0.parse()
    }
}

/// A role.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub description: Cow<'static, str>,
    pub created_at: DateTime<Utc>,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<Role> for crate::api::common::Role {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.into(),
            description: role.description.into(),
            created_at: role.created_at,
        }
    }
}

impl TryFrom<crate::api::common::Role> for Role {
    type Error = Error;

    fn try_from(role: crate::api::common::Role) -> Result<Self> {
        Ok(Self {
            id: role.id.try_into()?,
            description: role.description.into(),
            created_at: role.created_at,
        })
    }
}

/// A set of roles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleList(pub(crate) Vec<Role>);

impl RoleList {
    /// Create a new role list from the list of built-in roles.
    pub fn new_built_in() -> Self {
        Self(Role::BUILT_INS.iter().copied().cloned().collect())
    }

    /// Move all the elements from `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut RoleList) {
        self.0.extend(other.0.iter().cloned());
    }
}

impl From<RoleList> for crate::api::common::RoleList {
    fn from(role_list: RoleList) -> Self {
        Self(role_list.0.into_iter().map(Into::into).collect())
    }
}

impl<S: std::hash::BuildHasher + Default> From<RoleList> for HashMap<RoleId, Role, S> {
    fn from(role_list: RoleList) -> Self {
        role_list
            .0
            .into_iter()
            .map(|role| (role.id.clone(), role))
            .collect()
    }
}

impl TryFrom<crate::api::common::RoleList> for RoleList {
    type Error = Error;

    fn try_from(role_list: crate::api::common::RoleList) -> Result<Self> {
        role_list
            .0
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Role>>>()
            .map(RoleList)
    }
}

impl FromIterator<Role> for RoleList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Role>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for RoleList {
    type Item = Role;
    type IntoIter = std::vec::IntoIter<Role>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
