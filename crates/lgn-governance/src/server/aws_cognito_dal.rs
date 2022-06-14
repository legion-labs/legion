use std::{borrow::Cow, collections::HashMap};

use aws_sdk_cognitoidentityprovider::Region;
use chrono::{TimeZone, Utc};

use crate::types::UserInfo;

use super::{Error, Result};

pub struct AwsCognitoDal {
    pub region: Region,
    pub user_pool_id: String,
    pub client: aws_sdk_cognitoidentityprovider::Client,
}

impl AwsCognitoDal {
    pub async fn new(
        region: Option<impl Into<Cow<'static, str>>>,
        user_pool_id: impl Into<String>,
    ) -> Result<Self> {
        let config = aws_config::from_env();

        let config = if let Some(region) = region {
            let region = Region::new(region);

            config.region(region)
        } else {
            config
        }
        .load()
        .await;

        let region = config
            .region()
            .ok_or_else(|| Error::Configuration("no AWS region was defined".to_string()))?
            .clone();

        let user_pool_id = user_pool_id.into();

        if user_pool_id.is_empty() {
            return Err(Error::Configuration(
                "no AWS Cognito user pool id was defined".to_string(),
            ));
        }

        let client = aws_sdk_cognitoidentityprovider::Client::new(&config);

        Ok(Self {
            region,
            user_pool_id,
            client,
        })
    }

    pub async fn get_user_info(&self, username: &str) -> Result<UserInfo> {
        let resp = self
            .client
            .admin_get_user()
            .set_user_pool_id(Some(self.user_pool_id.clone()))
            .set_username(Some(username.to_string()))
            .send()
            .await
            .map_err::<aws_sdk_cognitoidentityprovider::Error, _>(Into::into)?;

        let attrs: HashMap<String, String> = resp
            .user_attributes
            .ok_or_else(|| Error::Unexpected("user has no attributes".to_string()))?
            .into_iter()
            .filter_map(|attr| match (attr.name, attr.value) {
                (Some(name), Some(value)) => Some((name, value)),
                _ => None,
            })
            .collect();

        let user_info = lgn_auth::UserInfo {
            sub: attrs
                .get("sub")
                .ok_or_else(|| Error::Unexpected("user has no sub".to_string()))?
                .to_string(),
            name: attrs.get("name").map(ToString::to_string),
            given_name: attrs.get("given_name").map(ToString::to_string),
            family_name: attrs.get("family_name").map(ToString::to_string),
            middle_name: attrs.get("middle_name").map(ToString::to_string),
            nickname: attrs.get("nickname").map(ToString::to_string),
            username: resp.username,
            preferred_username: attrs.get("preferred_username").map(ToString::to_string),
            profile: attrs.get("profile").map(ToString::to_string),
            picture: attrs.get("picture").map(ToString::to_string),
            website: attrs.get("website").map(ToString::to_string),
            email: attrs.get("email").map(ToString::to_string),
            email_verified: attrs.get("email_verified").map(|s| s == "true"),
            gender: attrs.get("gender").map(ToString::to_string),
            birthdate: attrs.get("birthdate").map(ToString::to_string),
            zoneinfo: attrs.get("zoneinfo").map(ToString::to_string),
            locale: attrs.get("locale").map(ToString::to_string),
            phone_number: attrs.get("phone_number").map(ToString::to_string),
            phone_number_verified: attrs.get("phone_number_verified").map(|s| s == "true"),
            updated_at: resp
                .user_last_modified_date
                .map(|d| Utc.timestamp_nanos(d.as_nanos() as i64).to_string()),
            azure_oid: attrs.get("custom:azure_oid").map(ToString::to_string),
            azure_tid: attrs.get("custom:azure_tid").map(ToString::to_string),
        };

        user_info.try_into().map_err(Into::into)
    }
}
