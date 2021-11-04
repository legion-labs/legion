use std::sync::Arc;

use legion_aws::lambda::api_gateway::{
    APIGatewayCustomAuthorizerRequest, APIGatewayCustomAuthorizerResponse, APIGatewayPolicyBuilder,
};
use log::info;

use lambda_runtime::Context;
use serde_json::json;

use legion_online::authentication::{
    jwt::{signature_validation::SignatureValidation, Token, Validation},
    UserInfo,
};

pub struct Handler<V> {
    validation: Arc<Validation<'static, V>>,
}

impl<V> Handler<V>
where
    V: SignatureValidation,
{
    pub fn new(validator: Arc<Validation<'static, V>>) -> Self {
        Self {
            validation: validator,
        }
    }

    pub async fn handle(
        &self,
        request: APIGatewayCustomAuthorizerRequest,
        _context: Context,
    ) -> anyhow::Result<APIGatewayCustomAuthorizerResponse> {
        info!("Client token: {}", request.authorization_token);
        info!("Method ARN: {}", request.method_arn.to_string());

        let token: Token = (&request.authorization_token[..]).try_into()?;
        let user_info: UserInfo = token.into_claims(&self.validation)?;

        let policy = APIGatewayPolicyBuilder::new(request.method_arn.base_method_arn)
            .allow_all_methods()
            .build();

        // new! -- add additional key-value pairs associated with the authenticated principal
        // these are made available by APIGW like so: $context.authorizer.<key>
        // additional context is cached
        Ok(APIGatewayCustomAuthorizerResponse {
            principal_id: user_info.sub,
            policy_document: policy,
            context: json!({
            "stringKey": "stringval",
            "numberKey": 123,
            "booleanKey": true
            }),
        })
    }
}
