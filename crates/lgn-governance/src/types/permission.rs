use itertools::Itertools;
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    fmt::{Display, Formatter},
    hash::Hash,
    str::FromStr,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "tabled")]
use tabled::Tabled;

use super::{Error, Result};

#[macro_export]
macro_rules! optional_permission_id {
    ($name:ident) => {
        Some(crate::types::PermissionId::$name)
    };
    () => {
        None
    };
}

#[macro_export]
macro_rules! declare_built_in_permissions {
    ($($name:ident$(($parent_name:ident))?: $description:literal), *$(,)?) => {
        impl crate::types::PermissionId {
            $(
            pub const $name: Self =
                crate::types::PermissionId(std::borrow::Cow::Borrowed(casey::lower!(stringify!($name))));
            )*

            const ALL_BY_NAME: &'static [(&'static str, &'static Self)] = &[
                $(
                    (casey::lower!(stringify!($name)), &Self::$name),
                )*
            ];

            pub(crate) fn get_built_in(s: &str) -> Option<&'static Self> {
                Self::ALL_BY_NAME.iter().find(|(name, _)| *name == s).map(|(_, id)| *id)
            }

            pub(crate) const BUILT_INS: &'static [&'static Self] = &[
                $(
                    &Self::$name,
                )*
            ];
        }

        impl crate::types::Permission {
            $(
            pub const $name: Self = Self {
                id: crate::types::PermissionId::$name,
                description: std::borrow::Cow::Borrowed($description),
                parent_id: crate::optional_permission_id!($($parent_name)?),
                created_at: chrono::MIN_DATETIME,
            };
            )*

            pub const BUILT_INS: &'static [&'static crate::types::Permission] = &[
                $(
                    &Self::$name,
                )*
            ];
        }
    };
}

/// A permission identifier.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
pub struct PermissionId(pub(crate) Cow<'static, str>);

impl Display for PermissionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PermissionId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidPermissionId(s.to_string()));
        }

        if s.contains(|c: char| !matches!(c, 'a'..='z' | '0'..='9' | '_')) {
            return Err(Error::InvalidPermissionId(s.to_string()));
        }

        Ok(if let Some(built_in) = PermissionId::get_built_in(s) {
            built_in.clone()
        } else {
            PermissionId(Cow::Owned(s.to_string()))
        })
    }
}

impl<'a> From<PermissionId> for crate::api::permission::PermissionId {
    fn from(permission_id: PermissionId) -> Self {
        Self(permission_id.0.to_string())
    }
}

impl<'a> TryFrom<crate::api::permission::PermissionId> for PermissionId {
    type Error = Error;

    fn try_from(permission_id: crate::api::permission::PermissionId) -> Result<Self> {
        permission_id.0.parse()
    }
}

/// A permission.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Permission {
    pub id: PermissionId,
    pub description: Cow<'static, str>,
    #[cfg_attr(
        feature = "tabled",
        tabled(display_with = "crate::formatter::optional")
    )]
    pub parent_id: Option<PermissionId>,
    pub created_at: DateTime<Utc>,
}

impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<Permission> for crate::api::permission::Permission {
    fn from(permission: Permission) -> Self {
        Self {
            id: permission.id.into(),
            description: permission.description.into(),
            parent_id: permission.parent_id.map(Into::into),
            created_at: permission.created_at,
        }
    }
}

impl TryFrom<crate::api::permission::Permission> for Permission {
    type Error = Error;

    fn try_from(permission: crate::api::permission::Permission) -> Result<Self> {
        Ok(Self {
            id: permission.id.try_into()?,
            description: permission.description.into(),
            parent_id: permission.parent_id.map(TryInto::try_into).transpose()?,
            created_at: permission.created_at,
        })
    }
}

/// A set of permissions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionList(pub(crate) Vec<Permission>);

impl PermissionList {
    pub fn iter(&self) -> impl Iterator<Item = &Permission> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Permission> {
        self.0.iter_mut()
    }
}

impl From<PermissionList> for crate::api::permission::PermissionList {
    fn from(permission_list: PermissionList) -> Self {
        Self(permission_list.0.into_iter().map(Into::into).collect())
    }
}

impl From<PermissionList> for PermissionSet {
    fn from(permission_list: PermissionList) -> Self {
        PermissionSet(
            permission_list
                .0
                .into_iter()
                .map(|permission| permission.id)
                .collect(),
        )
    }
}

impl<S: std::hash::BuildHasher + Default> From<PermissionList>
    for HashMap<PermissionId, Permission, S>
{
    fn from(permission_list: PermissionList) -> Self {
        permission_list
            .0
            .into_iter()
            .map(|permission| (permission.id.clone(), permission))
            .collect()
    }
}

impl TryFrom<crate::api::permission::PermissionList> for PermissionList {
    type Error = Error;

    fn try_from(permission_list: crate::api::permission::PermissionList) -> Result<Self> {
        permission_list
            .0
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Permission>>>()
            .map(PermissionList)
    }
}

impl FromIterator<Permission> for PermissionList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Permission>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for PermissionList {
    type Item = Permission;
    type IntoIter = std::vec::IntoIter<Permission>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A set of permissions.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionSet(pub(crate) BTreeSet<PermissionId>);

impl Display for PermissionSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

impl PermissionSet {
    /// Create a new permission set from the list of built-in permissions.
    pub fn new_built_in() -> Self {
        Self(PermissionId::BUILT_INS.iter().copied().cloned().collect())
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a permission by its identifier.
    pub fn contains(&self, permission_id: &PermissionId) -> bool {
        self.0.contains(permission_id)
    }

    /// Insert a permission.
    ///
    /// Returns `true` if the permission was inserted, `false` if it was already
    /// present.
    pub fn insert(&mut self, permission_id: PermissionId) -> bool {
        self.0.insert(permission_id)
    }

    /// Remove a permission.
    ///
    /// Returns `true` if the permission was removed, `false` if it was not
    pub fn remove(&mut self, permission_id: &PermissionId) -> bool {
        self.0.remove(permission_id)
    }

    /// Extend the set with the given permission set.
    pub fn extend(&mut self, iter: impl IntoIterator<Item = PermissionId>) {
        self.0.extend(iter);
    }
}

impl IntoIterator for PermissionSet {
    type Item = PermissionId;
    type IntoIter = std::collections::btree_set::IntoIter<PermissionId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<PermissionId> for PermissionSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = PermissionId>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a PermissionSet {
    type Item = &'a PermissionId;
    type IntoIter = std::collections::btree_set::Iter<'a, PermissionId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
