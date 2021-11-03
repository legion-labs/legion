use std::{net::SocketAddr, sync::Arc};

use anyhow::bail;
use async_trait::async_trait;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use log::{debug, info, warn};
use tokio::sync::Mutex;
use url::Url;

use super::Authenticator;

pub struct AwsCognitoClientAuthenticator {
    pub domain_name: String,
    pub region: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub port: u16,
    pub identity_provider: Option<String>,
}

use super::ClientTokenSet;
use super::UserInfo;

/// An `AwsCognitoClientAuthenticator`'s primary goal is to authenticate a user and return a
/// `ClientTokenSet` containing the user's id, access and refresh tokens.
///
/// It can do so by:
/// - Authenticating the user interactively with the identity provider in a web-browser.
/// - Exchanging a refresh token for a new access token.
///
/// It can also validate a user's access token and return a `UserInfo` struct containing the user's
/// profile.
impl AwsCognitoClientAuthenticator {
    /// Creates an authenticator from a valid AWS Cognito URL.
    ///
    /// # Example
    ///
    /// ```
    /// use legion_online::authentication::AwsCognitoClientAuthenticator;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http://localhost:5001/&identity_provider=Azure").unwrap();
    /// let auth = AwsCognitoClientAuthenticator::from_authorization_url(&url).unwrap();
    /// ```
    pub fn from_authorization_url(authorization_url: &Url) -> anyhow::Result<Self> {
        let host_parts = authorization_url
            .host()
            .ok_or_else(|| anyhow::anyhow!("no host in URL"))?
            .to_string()
            .split('.')
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();

        if host_parts.len() != 5 {
            bail!("host must respect the `<domain_name>.auth.<region>.amazoncognito.com` format");
        }

        let domain_name = host_parts[0].clone();
        let region = host_parts[2].clone();

        if authorization_url.path() != "/oauth2/authorize" {
            bail!("URL must be an AWS Cognito authorization URL");
        }

        let client_id = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "client_id")
            .map(|(_, v)| v.to_string())
            .expect("no client_id in URL");

        let scopes: Vec<String> = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "scope")
            .map(|(_, v)| v.split('+').map(std::string::ToString::to_string).collect())
            .expect("no scope in URL");

        let redirect_uri: Url = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "redirect_uri")
            .map(|(_, v)| v.to_string())
            .expect("no redirect_uri in URL")
            .parse()?;

        let identity_provider: Option<String> = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "identity_provider")
            .map(|(_, v)| v.to_string());

        if redirect_uri.scheme() != "http" {
            anyhow::bail!("redirect_uri must use the `http` scheme");
        }

        match redirect_uri.host() {
            Some(url::Host::Domain("localhost")) => {}
            Some(_) => anyhow::bail!("redirect_uri must use the `localhost` host"),
            None => anyhow::bail!("redirect_uri must have a host"),
        };

        // If there is no explicit port, assume the default port for the scheme.
        let port = redirect_uri.port().unwrap_or(80);

        Ok(Self {
            domain_name,
            region,
            client_id,
            scopes,
            port,
            identity_provider,
        })
    }

    fn get_callback_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.port))
    }

    fn get_redirect_uri(&self) -> String {
        format!("http://localhost:{}/", self.port)
    }

    fn get_base_url(&self, path: &str) -> Url {
        Url::parse(&format!(
            "https://{}.auth.{}.amazoncognito.com/{}",
            self.domain_name, self.region, path
        ))
        .unwrap()
    }

    /// Get the authorization URL.
    ///
    /// # Example
    ///
    /// ```
    /// use legion_online::authentication::AwsCognitoClientAuthenticator;
    ///
    /// # fn main() {
    /// let auth = AwsCognitoClientAuthenticator{
    ///     domain_name: "legionlabs-playground".to_string(),
    ///     region: "ca-central-1".to_string(),
    ///     client_id: "4a6vcgqr108in51n3di730hk25".to_string(),
    ///     scopes: vec![
    ///         "aws.cognito.signin.user.admin".to_string(),
    ///         "email".to_string(),
    ///         "openid".to_string(),
    ///     ],
    ///     port: 5001,
    ///     identity_provider: Some("Azure".to_string()),
    /// };
    ///
    /// assert_eq!(
    ///     auth.get_authorization_url().as_str(),
    ///     "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin%2Bemail%2Bopenid&redirect_uri=http%3A%2F%2Flocalhost%3A5001%2F&identity_provider=Azure",
    /// );
    /// # }
    /// ```
    pub fn get_authorization_url(&self) -> String {
        let mut url = self.get_base_url("oauth2/authorize");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.scopes.join("+"))
            .append_pair("redirect_uri", &self.get_redirect_uri());

        if let Some(identity_provider) = &self.identity_provider {
            url.query_pairs_mut()
                .append_pair("identity_provider", identity_provider);
        }

        url.to_string()
    }

    /// Get the logout URL.
    ///
    /// # Example
    ///
    /// ```
    /// use legion_online::authentication::AwsCognitoClientAuthenticator;
    ///
    /// # fn main() {
    /// let auth = AwsCognitoClientAuthenticator{
    ///     domain_name: "legionlabs-playground".to_string(),
    ///     region: "ca-central-1".to_string(),
    ///     client_id: "4a6vcgqr108in51n3di730hk25".to_string(),
    ///     scopes: vec![
    ///         "aws.cognito.signin.user.admin".to_string(),
    ///         "email".to_string(),
    ///         "openid".to_string(),
    ///     ],
    ///     port: 5001,
    ///     identity_provider: Some("Azure".to_string()),
    /// };
    ///
    /// assert_eq!(
    ///     auth.get_logout_url().as_str(),
    ///     "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/logout?client_id=4a6vcgqr108in51n3di730hk25&redirect_uri=http%3A%2F%2Flocalhost%3A5001%2F",
    /// );
    /// # }
    /// ```
    pub fn get_logout_url(&self) -> String {
        let mut url = self.get_base_url("logout");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.get_redirect_uri());

        url.to_string()
    }

    /// Get the access token URL.
    ///
    /// # Example
    ///
    /// ```
    /// use legion_online::authentication::AwsCognitoClientAuthenticator;
    ///
    /// # fn main() {
    /// let auth = AwsCognitoClientAuthenticator{
    ///     domain_name: "legionlabs-playground".to_string(),
    ///     region: "ca-central-1".to_string(),
    ///     client_id: "4a6vcgqr108in51n3di730hk25".to_string(),
    ///     scopes: vec![
    ///         "aws.cognito.signin.user.admin".to_string(),
    ///         "email".to_string(),
    ///         "openid".to_string(),
    ///     ],
    ///     port: 5001,
    ///     identity_provider: Some("Azure".to_string()),
    /// };
    ///
    /// assert_eq!(
    ///     auth.get_access_token_url().as_str(),
    ///     "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/token",
    /// );
    /// # }
    /// ```
    pub fn get_access_token_url(&self) -> String {
        self.get_base_url("oauth2/token").to_string()
    }

    /// Get the user info URL.
    ///
    /// # Example
    ///
    /// ```
    /// use legion_online::authentication::AwsCognitoClientAuthenticator;
    ///
    /// # fn main() {
    /// let auth = AwsCognitoClientAuthenticator{
    ///     domain_name: "legionlabs-playground".to_string(),
    ///     region: "ca-central-1".to_string(),
    ///     client_id: "4a6vcgqr108in51n3di730hk25".to_string(),
    ///     scopes: vec![
    ///         "aws.cognito.signin.user.admin".to_string(),
    ///         "email".to_string(),
    ///         "openid".to_string(),
    ///     ],
    ///     port: 5001,
    ///     identity_provider: Some("Azure".to_string()),
    /// };
    ///
    /// assert_eq!(
    ///     auth.get_user_info_url().as_str(),
    ///     "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/userInfo",
    /// );
    /// # }
    /// ```
    pub fn get_user_info_url(&self) -> String {
        self.get_base_url("oauth2/userInfo").to_string()
    }

    async fn receive_authorization_code(&self) -> anyhow::Result<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        let make_service = make_service_fn(move |socket: &AddrStream| {
            let tx = Arc::clone(&tx);
            debug!("new connection from: {}", socket.remote_addr());

            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let tx = Arc::clone(&tx);

                    async move {
                        debug!("received callback request: {:?}", req);

                        // We only accept calls to the root path.
                        if req.uri().path() != "/" {
                            warn!(
                                "rejecting request on unsupported path: {}",
                                req.uri().path()
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::empty())?);
                        }

                        // Only GETs are valid.
                        if req.method() != hyper::Method::GET {
                            warn!(
                                "rejecting request with not allowed method: {}",
                                req.method(),
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::METHOD_NOT_ALLOWED)
                                .body(Body::empty())?);
                        }

                        // Find the code parameter.
                        let code = req
                            .uri()
                            .query()
                            .map(|v| {
                                url::form_urlencoded::parse(v.as_bytes())
                                    .into_owned()
                                    .find(|(k, _)| k == "code")
                                    .map(|(_, code)| code)
                            })
                            .expect("failed to parse query string");

                        let code = if let Some(code) = code {
                            code
                        } else {
                            warn!("rejecting request without code");

                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::empty())?);
                        };

                        if let Some(tx) = tx.lock().await.take() {
                            let _err = tx.send(code);

                            info!("authentication succeeded");

                            Ok::<_, anyhow::Error>(Response::new(Body::from(include_str!(
                                "static/authentication_succeeded.html"
                            ))))
                        } else {
                            warn!("ignoring second successful authentication call");

                            Ok(Response::builder()
                                .status(StatusCode::PRECONDITION_FAILED)
                                .body(Body::empty())?)
                        }
                    }
                }))
            }
        });

        let mut code = String::default();

        let server = Server::bind(&self.get_callback_addr())
            .serve(make_service)
            .with_graceful_shutdown(async {
                code = rx.await.unwrap();
                info!(
                    "received authorization code `{}`: shutting down temporary HTTP server",
                    code
                );
            });

        if let Err(e) = server.await {
            bail!("failed to serve callback server: {}", e);
        }

        Ok(code)
    }

    async fn receive_logout_confirmation(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        let make_service = make_service_fn(move |socket: &AddrStream| {
            let tx = Arc::clone(&tx);
            debug!("new connection from: {}", socket.remote_addr());

            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let tx = Arc::clone(&tx);

                    async move {
                        debug!("received callback request: {:?}", req);

                        // We only accept calls to the root path.
                        if req.uri().path() != "/" {
                            warn!(
                                "rejecting request on unsupported path: {}",
                                req.uri().path()
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::empty())?);
                        }

                        // Only GETs are valid.
                        if req.method() != hyper::Method::GET {
                            warn!(
                                "rejecting request with not allowed method: {}",
                                req.method(),
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::METHOD_NOT_ALLOWED)
                                .body(Body::empty())?);
                        }

                        if let Some(tx) = tx.lock().await.take() {
                            let _err = tx.send(());

                            info!("logout succeeded");

                            Ok::<_, anyhow::Error>(Response::new(Body::from(include_str!(
                                "static/logout_succeeded.html"
                            ))))
                        } else {
                            warn!("ignoring second successful logout call");

                            Ok(Response::builder()
                                .status(StatusCode::PRECONDITION_FAILED)
                                .body(Body::empty())?)
                        }
                    }
                }))
            }
        });

        let server = Server::bind(&self.get_callback_addr())
            .serve(make_service)
            .with_graceful_shutdown(async {
                rx.await.unwrap();
                info!("received logout confirmation: shutting down temporary HTTP server");
            });

        if let Err(e) = server.await {
            bail!("failed to serve callback server: {}", e);
        }

        Ok(())
    }

    /// Get the authorization code by opening an interactive browser window.
    async fn get_authorization_code_interactive(&self) -> anyhow::Result<String> {
        let authorization_url = self.get_authorization_url();

        info!("Opening web-browser at: {}", authorization_url);

        webbrowser::open(authorization_url.as_str())?;

        self.receive_authorization_code().await
    }

    /// Get a token set from an authorization code.
    async fn get_token_set_from_authorization_code(
        &self,
        code: &str,
    ) -> anyhow::Result<ClientTokenSet> {
        let client =
            hyper::Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());

        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(self.get_access_token_url().as_str())
            .header(
                hyper::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );

        let req = req.body(Body::from(format!(
            "grant_type=authorization_code&client_id={}&code={}&redirect_uri={}",
            self.client_id,
            code,
            self.get_redirect_uri(),
        )))?;

        let resp = client.request(req).await?;
        let bytes = hyper::body::to_bytes(resp.into_body()).await?;
        serde_json::from_slice(&bytes).map_err(Into::into)
    }

    /// Get user information from an access token.
    pub async fn get_user_info(&self, access_token: &str) -> anyhow::Result<UserInfo> {
        let client =
            hyper::Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());

        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(self.get_user_info_url().as_str())
            .header(
                hyper::header::AUTHORIZATION,
                format!("Bearer {}", access_token),
            )
            .body(Body::empty())?;

        let resp = client.request(req).await?;
        let bytes = hyper::body::to_bytes(resp.into_body()).await?;
        serde_json::from_slice(&bytes).map_err(Into::into)
    }
}

#[async_trait]
impl Authenticator for AwsCognitoClientAuthenticator {
    /// Get a token set by opening a possible interactive browser window.
    async fn login(&self) -> anyhow::Result<ClientTokenSet> {
        let code = self.get_authorization_code_interactive().await?;

        self.get_token_set_from_authorization_code(&code).await
    }

    /// Get a token set from a refresh token.
    ///
    /// If the call does not return a new refresh token within the `TokenSet`, the specified
    /// refresh token will be filled in instead.
    async fn refresh_login(&self, refresh_token: &str) -> anyhow::Result<ClientTokenSet> {
        let client =
            hyper::Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());

        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(self.get_access_token_url().as_str())
            .header(
                hyper::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );

        let req = req.body(Body::from(format!(
            "grant_type=refresh_token&client_id={}&refresh_token={}",
            self.client_id, refresh_token,
        )))?;

        let resp = client.request(req).await?;
        let bytes = hyper::body::to_bytes(resp.into_body()).await?;
        serde_json::from_slice(&bytes)
            .map_err(Into::into)
            .map(|mut token_set: ClientTokenSet| {
                if token_set.refresh_token.is_none() {
                    token_set.refresh_token = Some(refresh_token.to_owned());
                }

                token_set
            })
    }

    /// Logout by opening an interactive browser window.
    async fn logout(&self) -> anyhow::Result<()> {
        let logout_url = self.get_logout_url();

        info!("Opening web-browser at: {}", logout_url);

        webbrowser::open(logout_url.as_str())?;

        self.receive_logout_confirmation().await
    }
}
