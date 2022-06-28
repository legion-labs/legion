use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use super::{Error, Result};

#[cfg(feature = "tabled")]
use tabled::Tabled;

/// An extended user identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ExtendedUserId {
    UserId(UserId),
    Email(String),
    Alias(UserAlias),
    MySelf,
}

impl Display for ExtendedUserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserId(user_id) => write!(f, "{}", user_id),
            Self::Email(email) => write!(f, "@{}", email),
            Self::Alias(user_alias) => write!(f, "@{}", user_alias),
            Self::MySelf => write!(f, "@me"),
        }
    }
}

impl FromStr for ExtendedUserId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidUserId(s.to_string()));
        }

        Ok(if let Some(s) = s.strip_prefix('@') {
            if s == "me" {
                Self::MySelf
            } else if s.contains('@') {
                Self::Email(s.to_string())
            } else {
                Self::Alias(s.parse()?)
            }
        } else {
            Self::UserId(s.parse()?)
        })
    }
}

impl From<UserId> for ExtendedUserId {
    fn from(user_id: UserId) -> Self {
        Self::UserId(user_id)
    }
}

impl From<ExtendedUserId> for crate::api::user::ExtendedUserId {
    fn from(user_id: ExtendedUserId) -> Self {
        Self(user_id.to_string())
    }
}

impl TryFrom<crate::api::user::ExtendedUserId> for ExtendedUserId {
    type Error = Error;

    fn try_from(user_id: crate::api::user::ExtendedUserId) -> Result<Self> {
        user_id.0.parse()
    }
}

/// A user identifier.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct UserId(String);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidUserId(s.to_string()));
        }

        Ok(Self(s.to_string()))
    }
}

impl From<UserId> for crate::api::user::UserId {
    fn from(user_id: UserId) -> Self {
        Self(user_id.0)
    }
}

impl From<crate::api::user::UserId> for UserId {
    fn from(user_id: crate::api::user::UserId) -> Self {
        Self(user_id.0)
    }
}

impl<'s> PartialEq<&'s str> for UserId {
    fn eq(&self, other: &&'s str) -> bool {
        self.0 == *other
    }
}

/// User information.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct UserInfo {
    pub id: UserId,
    pub name: String,
    pub family_name: String,
    pub middle_name: String,
    pub given_name: String,
    pub email: String,
}

impl TryFrom<lgn_auth::UserInfo> for UserInfo {
    type Error = Error;

    fn try_from(user_info: lgn_auth::UserInfo) -> Result<Self> {
        Ok(Self {
            id: user_info
                .username
                .as_ref()
                .ok_or_else(|| Error::Unexpected("missing `username` in user info while converting from `lgn_auth::UserInfo` to `types::UserInfo`".to_string()))?
                .parse()?,
            name: user_info.name(),
            family_name: user_info.family_name.unwrap_or_default(),
            middle_name: user_info.middle_name.unwrap_or_default(),
            given_name: user_info.given_name.unwrap_or_default(),
            email: user_info.email.unwrap_or_default(),
        })
    }
}

impl From<UserInfo> for crate::api::user::UserInfo {
    fn from(user_info: UserInfo) -> Self {
        Self {
            id: user_info.id.into(),
            name: user_info.name,
            family_name: user_info.family_name,
            middle_name: user_info.middle_name,
            given_name: user_info.given_name,
            email: user_info.email,
        }
    }
}

impl From<crate::api::user::UserInfo> for UserInfo {
    fn from(user_info: crate::api::user::UserInfo) -> Self {
        Self {
            id: user_info.id.into(),
            name: user_info.name,
            family_name: user_info.family_name,
            middle_name: user_info.middle_name,
            given_name: user_info.given_name,
            email: user_info.email,
        }
    }
}

/// A user alias.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct UserAlias(String);

impl Display for UserAlias {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UserAlias {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() || s.contains('@') {
            return Err(Error::InvalidUserAlias(s.to_string()));
        }

        Ok(Self(s.to_string()))
    }
}

impl From<UserAlias> for crate::api::user::UserAlias {
    fn from(user_alias: UserAlias) -> Self {
        Self(user_alias.0)
    }
}

impl From<crate::api::user::UserAlias> for UserAlias {
    fn from(user_alias: crate::api::user::UserAlias) -> Self {
        Self(user_alias.0)
    }
}

impl<'s> PartialEq<&'s str> for UserAlias {
    fn eq(&self, other: &&'s str) -> bool {
        self.0 == *other
    }
}

/// A user alias.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct UserAliasAssociation {
    pub alias: UserAlias,
    pub user_id: UserId,
}

impl From<UserAliasAssociation> for crate::api::user::UserAliasAssociation {
    fn from(user_alias_association: UserAliasAssociation) -> Self {
        Self {
            alias: user_alias_association.alias.into(),
            user_id: user_alias_association.user_id.into(),
        }
    }
}

impl From<crate::api::user::UserAliasAssociation> for UserAliasAssociation {
    fn from(user_alias_association: crate::api::user::UserAliasAssociation) -> Self {
        Self {
            alias: user_alias_association.alias.into(),
            user_id: user_alias_association.user_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_user_id_parse() {
        let extended_user_id = ExtendedUserId::from_str("abc").unwrap();
        assert_eq!(
            extended_user_id,
            ExtendedUserId::UserId(UserId("abc".to_string()))
        );

        let extended_user_id = ExtendedUserId::from_str("@bob@aol.com").unwrap();
        assert_eq!(
            extended_user_id,
            ExtendedUserId::Email("bob@aol.com".to_string())
        );

        let extended_user_id = ExtendedUserId::from_str("@bob").unwrap();
        assert_eq!(
            extended_user_id,
            ExtendedUserId::Alias(UserAlias("bob".to_string()))
        );

        let extended_user_id = ExtendedUserId::from_str("@me").unwrap();
        assert_eq!(extended_user_id, ExtendedUserId::MySelf,);
    }
}
