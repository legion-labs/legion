use std::sync::Arc;

use anyhow::bail;
use legion_aws::lambda::api_gateway::{
    APIGatewayCustomAuthorizerRequest, APIGatewayCustomAuthorizerResponse, APIGatewayPolicyBuilder,
};
use log::{error, info};

use lambda_runtime::Context;
use serde_json::json;

pub struct Handler {
    validator: Arc<legion_auth::Validator>,
}

impl Handler {
    pub fn new(validator: Arc<legion_auth::Validator>) -> Self {
        Self { validator }
    }

    pub async fn handle(
        &self,
        request: APIGatewayCustomAuthorizerRequest,
        _context: Context,
    ) -> anyhow::Result<APIGatewayCustomAuthorizerResponse> {
        info!("Client token: {}", request.authorization_token);
        info!("Method ARN: {}", request.method_arn.to_string());

        let header = match jsonwebtoken::decode_header(&request.authorization_token) {
            Ok(header) => header,
            Err(err) => {
                error!("Error decoding JWT header: {}", err);

                bail!("Invalid token");
            }
        };

        let kid = match header.kid {
            Some(kid) => kid,
            None => {
                error!("No kid in JWT header");

                bail!("Invalid token");
            }
        };

        info!("Key identifier (kid): {}", kid);
        info!("Algorithm: {:?}", header.alg);

        let user_info = self
            .validator
            .validate(&kid, &request.authorization_token)
            .await?;

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
