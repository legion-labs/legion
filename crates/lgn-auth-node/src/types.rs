// TODO: Ideally we should use napi in lgn-auth and have a "node" feature.
// Unfortunately it led to many errors with the ci so we temporarily duplicate the struct.
// This has not impact on the generated TS code since TS is structurally typed anyway.

// Napi seems to cause this issue with clippy where it complains about
// the struct `UserInfo` not to be defined as `Self`.
#![allow(clippy::use_self)]

use lgn_auth::UserInfo as OriginalUserInfo;
use napi_derive::napi;

#[napi(object)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub middle_name: Option<String>,
    pub nickname: Option<String>,
    pub username: Option<String>,
    pub preferred_username: Option<String>,
    pub profile: Option<String>,
    pub picture: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    pub zoneinfo: Option<String>,
    pub locale: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_verified: Option<bool>,
    pub updated_at: Option<String>,
    pub azure_oid: Option<String>,
    pub azure_tid: Option<String>,
}

impl From<OriginalUserInfo> for UserInfo {
    fn from(original_user_info: OriginalUserInfo) -> Self {
        Self {
            sub: original_user_info.sub,
            name: original_user_info.name,
            given_name: original_user_info.given_name,
            family_name: original_user_info.family_name,
            middle_name: original_user_info.middle_name,
            nickname: original_user_info.nickname,
            username: original_user_info.username,
            preferred_username: original_user_info.preferred_username,
            profile: original_user_info.profile,
            picture: original_user_info.picture,
            website: original_user_info.website,
            email: original_user_info.email,
            email_verified: original_user_info.email_verified,
            gender: original_user_info.gender,
            birthdate: original_user_info.birthdate,
            zoneinfo: original_user_info.zoneinfo,
            locale: original_user_info.locale,
            phone_number: original_user_info.phone_number,
            phone_number_verified: original_user_info.phone_number_verified,
            updated_at: original_user_info.updated_at,
            azure_oid: original_user_info.azure_oid,
            azure_tid: original_user_info.azure_tid,
        }
    }
}
