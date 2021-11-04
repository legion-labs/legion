use std::sync::Arc;

use anyhow::Context;
use legion_aws::lambda::run_lambda;
use legion_online::authentication::jwt::{
    signature_validation::AwsCognitoSignatureValidation, Validation,
};

mod handler;

use handler::Handler;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    simple_logger::init_with_level(log::Level::Info)?;

    let region = std::env::var("AWS_REGION").context("`AWS_REGION` is not set")?;
    let user_pool_id = std::env::var("AWS_COGNITO_USER_POOL_ID")
        .context("`AWS_COGNITO_USER_POOL_ID` is not set")?;
    //let editor_client_id = std::env::var("AWS_COGNITO_EDITOR_CLIENT_ID")
    //    .context("`AWS_COGNITO_EDITOR_CLIENT_ID` is not set")?;

    let validation = Arc::new(
        Validation::new(AwsCognitoSignatureValidation::new(&region, &user_pool_id).await?), //TODO: Fix this: .with_aud(&editor_client_id),
    );

    let handler = lambda_runtime::handler_fn(|request, context| async {
        let handler = Handler::new(Arc::clone(&validation));
        handler.handle(request, context).await
    });

    run_lambda(handler).await
}
