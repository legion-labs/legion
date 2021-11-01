use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIGatewayCustomAuthorizerRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub authorization_token: String,
    pub method_arn: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIGatewayCustomAuthorizerResponse {
    pub principal_id: String,
    pub policy_document: APIGatewayCustomAuthorizerPolicy,
    pub context: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct APIGatewayCustomAuthorizerPolicy {
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Statement")]
    statement: Vec<IAMPolicyStatement>,
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

/// Helps to build IAM policy statements.
///
/// # Example
///
/// ```
/// use lambda_api_gateway::iam::{Effect, IAMPolicyStatementBuilder};
pub struct APIGatewayPolicyBuilder {
    region: String,
    aws_account_id: String,
    rest_api_id: String,
    stage: String,
    policy: APIGatewayCustomAuthorizerPolicy,
}

#[derive(Serialize, Deserialize)]
pub enum Method {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "*PUT")]
    Put,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "OPTIONS")]
    Options,
    #[serde(rename = "*")]
    All,
}

#[derive(Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

static POLICY_VERSION: &str = "2012-10-17"; // override if necessary

impl APIGatewayPolicyBuilder {
    pub fn new(region: &str, account_id: &str, api_id: &str, stage: &str) -> Self {
        Self {
            region: region.to_string(),
            aws_account_id: account_id.to_string(),
            rest_api_id: api_id.to_string(),
            stage: stage.to_string(),
            policy: APIGatewayCustomAuthorizerPolicy {
                version: POLICY_VERSION.to_string(),
                statement: vec![],
            },
        }
    }

    pub fn add_method<T: Into<String>>(
        mut self,
        effect: Effect,
        method: &Method,
        resource: T,
    ) -> Self {
        let resource_arn = format!(
            "arn:aws:execute-api:{}:{}:{}/{}/{}/{}",
            &self.region,
            &self.aws_account_id,
            &self.rest_api_id,
            &self.stage,
            serde_json::to_string(&method).unwrap(),
            resource.into().trim_start_matches('/')
        );

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
