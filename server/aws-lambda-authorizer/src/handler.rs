use legion_aws::lambda::api_gateway::{
    APIGatewayCustomAuthorizerRequest, APIGatewayCustomAuthorizerResponse, APIGatewayPolicyBuilder,
};
use log::info;

use lambda_runtime::{error::HandlerError, Context};
use serde_json::json;

pub fn handler(
    request: APIGatewayCustomAuthorizerRequest,
    _context: Context,
) -> Result<APIGatewayCustomAuthorizerResponse, HandlerError> {
    info!("Client token: {}", request.authorization_token);
    info!("Method ARN: {}", request.method_arn);

    // TODO: Validate the JWT and extract the relevant information.
    let principal_id = "user|a1b2c3d4";

    // Toan send a 401 Unauthorized response to the client, we need to return:
    // Err(HandlerError{ msg: "Unauthorized".to_string(), backtrace: None });

    // if the token is valid, a policy must be generated which will allow or deny access to the client

    // if access is denied, the client will recieve a 403 Access Denied response
    // if access is allowed, API Gateway will proceed with the backend integration configured on the method that was called

    // this function must generate a policy that is associated with the recognized principal user identifier.
    // depending on your use case, you might store policies in a DB, or generate them on the fly

    // keep in mind, the policy is cached for 5 minutes by default (TTL is configurable in the authorizer)
    // and will apply to subsequent calls to any method/resource in the RestApi
    // made with the same token

    //the example policy below allows access to all resources in the RestApi
    let parts: Vec<&str> = request.method_arn.split(':').collect();
    let api_gateway_arn_tmp: Vec<&str> = parts[5].split('/').collect();
    let aws_account_id = parts[4];
    let region = parts[3];
    let rest_api_id = api_gateway_arn_tmp[0];
    let stage = api_gateway_arn_tmp[1];

    let policy = APIGatewayPolicyBuilder::new(region, aws_account_id, rest_api_id, stage)
        .allow_all_methods()
        .build();

    // new! -- add additional key-value pairs associated with the authenticated principal
    // these are made available by APIGW like so: $context.authorizer.<key>
    // additional context is cached
    Ok(APIGatewayCustomAuthorizerResponse {
        principal_id: principal_id.to_string(),
        policy_document: policy,
        context: json!({
        "stringKey": "stringval",
        "numberKey": 123,
        "booleanKey": true
        }),
    })
}
