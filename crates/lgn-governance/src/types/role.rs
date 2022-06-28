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

use super::{Error, PermissionSet, Result, SpaceId, UserId};

#[macro_export]
macro_rules! declare_built_in_roles {
    ($($name:ident: $description:literal => $($permission:ident)_*), *$(,)?) => {
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

            pub(crate) const BUILT_INS: &'static [&'static Self] = &[
                $(
                    &Self::$name,
                )*
            ];
        }

        lazy_static::lazy_static!{
            static ref STATIC_ROLES: std::collections::HashMap<crate::types::RoleId, crate::types::Role> = [
            $(
                crate::types::Role {
                    id: crate::types::RoleId::$name,
                    description: std::borrow::Cow::Borrowed($description),
                    permissions: [
                        $(
                            crate::types::PermissionId::$permission,
                        )*
                    ].iter().cloned().collect(),
                    created_at: chrono::MIN_DATETIME,
                },
            )*
            ].into_iter().map(|role| (role.id.clone(), role)).collect();
        }

        impl crate::types::Role {
            pub fn get_built_ins() -> [&'static Self; crate::types::RoleId::BUILT_INS.len()] {
                [
                    $(
                        STATIC_ROLES.get(&crate::types::RoleId::$name).unwrap()
                    )*
                ]
            }
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

        if s.contains(|c: char| !matches!(c, 'a'..='z' | '0'..='9' | '_')) {
            return Err(Error::InvalidRoleId(s.to_string()));
        }

        Ok(if let Some(built_in) = RoleId::get_built_in(s) {
            built_in.clone()
        } else {
            RoleId(Cow::Owned(s.to_string()))
        })
    }
}

impl<'a> From<RoleId> for crate::api::role::RoleId {
    fn from(role_id: RoleId) -> Self {
        Self(role_id.0.to_string())
    }
}

impl<'a> TryFrom<crate::api::role::RoleId> for RoleId {
    type Error = Error;

    fn try_from(role_id: crate::api::role::RoleId) -> Result<Self> {
        role_id.0.parse()
    }
}

/// A role.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Role {
    pub id: RoleId,
    pub description: Cow<'static, str>,
    pub created_at: DateTime<Utc>,
    pub permissions: PermissionSet,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<Role> for crate::api::role::Role {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.into(),
            description: role.description.into(),
            permissions: role.permissions.into_iter().map(Into::into).collect(),
            created_at: role.created_at,
        }
    }
}

impl TryFrom<crate::api::role::Role> for Role {
    type Error = Error;

    fn try_from(role: crate::api::role::Role) -> Result<Self> {
        Ok(Self {
            id: role.id.try_into()?,
            description: role.description.into(),
            permissions: role
                .permissions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
            created_at: role.created_at,
        })
    }
}

/// A set of roles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleList(pub(crate) Vec<Role>);

impl RoleList {
    pub fn iter(&self) -> impl Iterator<Item = &Role> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Role> {
        self.0.iter_mut()
    }
}

impl From<RoleList> for crate::api::role::RoleList {
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

impl TryFrom<crate::api::role::RoleList> for RoleList {
    type Error = Error;

    fn try_from(role_list: crate::api::role::RoleList) -> Result<Self> {
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

/// Defines the assignation of a role to a user, in an optional space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct RoleUserAssignation {
    pub user_id: UserId,
    pub role_id: RoleId,
    #[cfg_attr(
        feature = "tabled",
        tabled(display_with = "crate::formatter::optional")
    )]
    pub space_id: Option<SpaceId>,
}

impl TryFrom<crate::api::role::RoleUserAssignation> for RoleUserAssignation {
    type Error = Error;

    fn try_from(role_user_assignation: crate::api::role::RoleUserAssignation) -> Result<Self> {
        Ok(Self {
            user_id: role_user_assignation.user_id.into(),
            role_id: role_user_assignation.role_id.try_into()?,
            space_id: role_user_assignation.space_id.map(Into::into),
        })
    }
}

/// Defines the assignation of a role, in an optional space.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct RoleAssignation {
    pub role_id: RoleId,
    #[cfg_attr(
        feature = "tabled",
        tabled(display_with = "crate::formatter::optional")
    )]
    pub space_id: Option<SpaceId>,
}

impl RoleAssignation {
    pub fn into_role_user_assignation(self, user_id: UserId) -> RoleUserAssignation {
        RoleUserAssignation {
            user_id,
            role_id: self.role_id,
            space_id: self.space_id,
        }
    }
}

impl From<RoleUserAssignation> for RoleAssignation {
    fn from(role_user_assignation: RoleUserAssignation) -> Self {
        Self {
            role_id: role_user_assignation.role_id,
            space_id: role_user_assignation.space_id,
        }
    }
}

impl From<RoleAssignation> for crate::api::role::RoleAssignation {
    fn from(role_assignation: RoleAssignation) -> Self {
        Self {
            role_id: role_assignation.role_id.into(),
            space_id: role_assignation.space_id.map(Into::into),
        }
    }
}

impl TryFrom<crate::api::role::RoleAssignation> for RoleAssignation {
    type Error = Error;

    fn try_from(role_assignation: crate::api::role::RoleAssignation) -> Result<Self> {
        Ok(Self {
            role_id: role_assignation.role_id.try_into()?,
            space_id: role_assignation.space_id.map(Into::into),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoleAssignationPatch {
    pub set: BTreeSet<RoleAssignation>,
    pub unset: BTreeSet<RoleAssignation>,
}

impl RoleAssignationPatch {
    pub fn single_addition(role_id: RoleId, space_id: Option<SpaceId>) -> Self {
        Self {
            set: [RoleAssignation { role_id, space_id }]
                .into_iter()
                .collect(),
            unset: BTreeSet::new(),
        }
    }

    pub fn single_removal(role_id: RoleId, space_id: Option<SpaceId>) -> Self {
        Self {
            set: BTreeSet::new(),
            unset: [RoleAssignation { role_id, space_id }]
                .into_iter()
                .collect(),
        }
    }
}

impl From<RoleAssignationPatch> for crate::api::role::RoleAssignationPatch {
    fn from(role_assignation_patch: RoleAssignationPatch) -> Self {
        Self {
            set: role_assignation_patch
                .set
                .into_iter()
                .map(Into::into)
                .collect(),
            unset: role_assignation_patch
                .unset
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<crate::api::role::RoleAssignationPatch> for RoleAssignationPatch {
    type Error = Error;

    fn try_from(role_assignation_patch: crate::api::role::RoleAssignationPatch) -> Result<Self> {
        let unset = role_assignation_patch
            .unset
            .into_iter()
            .map(TryInto::try_into)
            .collect::<crate::types::Result<BTreeSet<RoleAssignation>>>()?;

        let set = role_assignation_patch
            .set
            .into_iter()
            .filter_map(|role_assignation| match role_assignation.try_into() {
                Ok(role_assignation) => {
                    // If the unset list contain the same role assignation, we
                    // remove it from the set to avoid useless work.
                    if unset.contains(&role_assignation) {
                        None
                    } else {
                        Some(Ok(role_assignation))
                    }
                }
                Err(err) => Some(Err(err)),
            })
            .collect::<crate::types::Result<BTreeSet<RoleAssignation>>>()?;

        Ok(Self { set, unset })
    }
}
