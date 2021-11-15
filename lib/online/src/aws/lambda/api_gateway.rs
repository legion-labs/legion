use std::str::FromStr;
use strum_macros::{Display, EnumString};

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIGatewayCustomAuthorizerRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub authorization_token: String,
    pub method_arn: APIGatewayMethodArn,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIGatewayCustomAuthorizerResponse {
    pub principal_id: String,
    pub policy_document: APIGatewayCustomAuthorizerPolicy,
    pub context: serde_json::Value,
}

/// The common attributes for all API Gateway method ARNs in a given authorizer.
#[derive(Debug, Clone, PartialEq)]
pub struct APIGatewayBaseMethodArn {
    pub region: String,
    pub aws_account_id: String,
    pub rest_api_id: String,
    pub stage: String,
}

/// An API Gateway method ARN.
///
/// This is a string representation of an ARN like: `arn:aws:execute-api:{regionId}:{accountId}:{apiId}/{stage}/{httpVerb}/[{resource}/[{child-resources}]]`
///
/// # Example
///
/// ```rust
/// use legion_aws::lambda::api_gateway::{APIGatewayBaseMethodArn, APIGatewayMethodArn, Method};
///
/// let api_gateway_method_arn_str = "arn:aws:execute-api:us-east-1:123456789012:api-id/dev/GET/users/{userId}/groups/{groupId}";
/// let api_gateway_method_arn: APIGatewayMethodArn = api_gateway_method_arn_str.parse().unwrap();
///     
/// assert_eq!(
///     APIGatewayMethodArn{
///         base_method_arn: APIGatewayBaseMethodArn{
///             region: "us-east-1".to_string(),
///             aws_account_id: "123456789012".to_string(),
///             rest_api_id: "api-id".to_string(),
///             stage: "dev".to_string(),
///         },
///         http_verb: Method::Get,
///         resource: "users/{userId}/groups/{groupId}".to_string(),
///     },
///     api_gateway_method_arn,
/// );
/// assert_eq!(api_gateway_method_arn.to_string(), api_gateway_method_arn_str);
///
/// // The type serializes and deserializes as a JSON string.
/// assert_eq!(
///     serde_json::to_string(&api_gateway_method_arn).unwrap(),
///     serde_json::to_string(&api_gateway_method_arn_str).unwrap(),
/// );
/// assert_eq!(
///     serde_json::from_str::<APIGatewayMethodArn>(&serde_json::to_string(&api_gateway_method_arn).unwrap()).unwrap(),
///     api_gateway_method_arn,
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct APIGatewayMethodArn {
    pub base_method_arn: APIGatewayBaseMethodArn,
    pub http_verb: Method,
    pub resource: String,
}

impl FromStr for APIGatewayMethodArn {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 6 {
            bail!("Invalid ARN: {}", s);
        }

        if parts[0] != "arn" {
            bail!("First part of the ARN must be `arn`: {}", s);
        }

        if parts[1] != "aws" {
            bail!("Second part of the ARN must be `aws`: {}", s);
        }

        if parts[2] != "execute-api" {
            bail!("Third part of the ARN must be `execute-api`: {}", s);
        }

        let region = parts[3].to_string();
        let aws_account_id = parts[4].to_string();

        let api_parts: Vec<&str> = parts[5].splitn(4, '/').collect();

        if api_parts.len() < 3 {
            bail!(
                "The API part of the ARN must have at least 3 elements: {}",
                parts[5],
            );
        }

        let rest_api_id = api_parts[0].to_string();
        let stage = api_parts[1].to_string();

        let base_method_arn = APIGatewayBaseMethodArn {
            region,
            aws_account_id,
            rest_api_id,
            stage,
        };

        let http_verb = api_parts[2].parse().unwrap();
        let resource = if api_parts.len() == 4 {
            api_parts[3]
        } else {
            ""
        }
        .to_string();

        Ok(Self {
            base_method_arn,
            http_verb,
            resource,
        })
    }
}

impl ToString for APIGatewayMethodArn {
    fn to_string(&self) -> String {
        format!(
            "arn:aws:execute-api:{}:{}:{}/{}/{}/{}",
            self.base_method_arn.region,
            self.base_method_arn.aws_account_id,
            self.base_method_arn.rest_api_id,
            self.base_method_arn.stage,
            self.http_verb,
            self.resource
        )
    }
}

impl<'de> Deserialize<'de> for APIGatewayMethodArn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for APIGatewayMethodArn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct APIGatewayCustomAuthorizerPolicy {
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Statement")]
    statement: Vec<IAMPolicyStatement>,
}

static POLICY_VERSION: &str = "2012-10-17"; // override if necessary

impl Default for APIGatewayCustomAuthorizerPolicy {
    fn default() -> Self {
        Self {
            version: POLICY_VERSION.to_string(),
            statement: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IAMPolicyStatement {
    #[serde(rename = "Action")]
    action: Vec<String>,
    #[serde(rename = "Effect")]
    effect: Effect,
    #[serde(rename = "Resource")]
    resource: Vec<String>,
}

/// An HTTP method.
///
/// # Example
///
/// ```rust
/// use legion_aws::lambda::api_gateway::Method;
///
/// let method: Method = "GET".parse().unwrap();
///
/// assert_eq!(method.to_string(), "GET");
/// ```
#[derive(Serialize, Deserialize, Display, EnumString, Debug, Clone, PartialEq)]
pub enum Method {
    #[strum(serialize = "GET")]
    #[serde(rename = "GET")]
    Get,
    #[strum(serialize = "POST")]
    #[serde(rename = "POST")]
    Post,
    #[strum(serialize = "PUT")]
    #[serde(rename = "PUT")]
    Put,
    #[strum(serialize = "DELETE")]
    #[serde(rename = "DELETE")]
    Delete,
    #[strum(serialize = "PATCH")]
    #[serde(rename = "PATCH")]
    Patch,
    #[strum(serialize = "HEAD")]
    #[serde(rename = "HEAD")]
    Head,
    #[strum(serialize = "OPTIONS")]
    #[serde(rename = "OPTIONS")]
    Options,
    #[strum(serialize = "*")]
    #[serde(rename = "*")]
    All,
}

#[derive(Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

/// Helps to build IAM policy statements.
///
/// # Example
///
/// ```
/// use legion_aws::lambda::api_gateway::{APIGatewayBaseMethodArn, APIGatewayPolicyBuilder};
///
/// let base_method_arn = APIGatewayBaseMethodArn{
///     region: "us-east-1".to_string(),
///     aws_account_id: "123456789012".to_string(),
///     rest_api_id: "api-id".to_string(),
///     stage: "dev".to_string(),
/// };
/// let policy = APIGatewayPolicyBuilder::new(base_method_arn)
///     .allow_all_methods()
///     .build();
///
/// assert_eq!(
///     serde_json::to_string(&policy).unwrap(),
///     r#"{"Version":"2012-10-17","Statement":[{"Action":["execute-api:Invoke"],"Effect":"Allow","Resource":["arn:aws:execute-api:us-east-1:123456789012:api-id/dev/*/*"]}]}"#,
/// );
/// ```
pub struct APIGatewayPolicyBuilder {
    base_method_arn: APIGatewayBaseMethodArn,
    policy: APIGatewayCustomAuthorizerPolicy,
}

impl APIGatewayPolicyBuilder {
    pub fn new(base_method_arn: APIGatewayBaseMethodArn) -> Self {
        Self {
            base_method_arn,
            policy: APIGatewayCustomAuthorizerPolicy::default(),
        }
    }

    pub fn new_from_method_arn(method_arn: &APIGatewayMethodArn, effect: Effect) -> Self {
        Self::new(method_arn.base_method_arn.clone()).add_method(
            effect,
            &method_arn.http_verb,
            method_arn.resource.clone(),
        )
    }

    pub fn add_method<T: Into<String>>(
        mut self,
        effect: Effect,
        method: &Method,
        resource: T,
    ) -> Self {
        let resource_arn = APIGatewayMethodArn {
            base_method_arn: self.base_method_arn.clone(),
            http_verb: method.clone(),
            resource: resource.into(),
        }
        .to_string();

        let stmt = IAMPolicyStatement {
            effect,
            action: vec!["execute-api:Invoke".to_string()],
            resource: vec![resource_arn],
        };

        self.policy.statement.push(stmt);
        self
    }

    pub fn allow_all_methods(self) -> Self {
        self.add_method(Effect::Allow, &Method::All, "*")
    }

    pub fn deny_all_methods(self) -> Self {
        self.add_method(Effect::Deny, &Method::All, "*")
    }

    pub fn allow_method(self, method: &Method, resource: String) -> Self {
        self.add_method(Effect::Allow, method, resource)
    }

    pub fn deny_method(self, method: &Method, resource: String) -> Self {
        self.add_method(Effect::Deny, method, resource)
    }

    // Creates and executes a new child thread.
    pub fn build(self) -> APIGatewayCustomAuthorizerPolicy {
        self.policy
    }
}
