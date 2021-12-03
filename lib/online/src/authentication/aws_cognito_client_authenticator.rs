use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use hyper::{
    client::HttpConnector,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server, StatusCode,
};
use hyper_rustls::HttpsConnector;
use log::{debug, info, warn};
use tokio::sync::{Mutex, Semaphore, SemaphorePermit};
use url::Url;

use super::{Authenticator, Error, Result};

pub struct AwsCognitoClientAuthenticator {
    pub domain_name: String,
    pub region: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub port: u16,
    pub identity_provider: Option<String>,

    client: Client<HttpsConnector<HttpConnector>>,
    semaphore: Semaphore,
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
    pub fn from_authorization_url(authorization_url: &Url) -> Result<Self> {
        let host_parts = authorization_url
            .host()
            .ok_or_else(|| Error::InvalidAuthorizationUrl("no host in URL".to_string()))?
            .to_string()
            .split('.')
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();

        if host_parts.len() != 5 {
            return Err(Error::InvalidAuthorizationUrl(
                "host must respect the `<domain_name>.auth.<region>.amazoncognito.com` format"
                    .to_string(),
            ));
        }

        let domain_name = host_parts[0].clone();
        let region = host_parts[2].clone();

        if authorization_url.path() != "/oauth2/authorize" {
            return Err(Error::InvalidAuthorizationUrl(
                "URL must be an AWS Cognito authorization URL".to_string(),
            ));
        }

        let client_id = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "client_id")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| Error::InvalidAuthorizationUrl("no client_id in URL".to_string()))?;

        let scopes: Vec<String> = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "scope")
            .map(|(_, v)| v.split('+').map(std::string::ToString::to_string).collect())
            .ok_or_else(|| Error::InvalidAuthorizationUrl("no scope in URL".to_string()))?;

        let redirect_uri: Url = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "redirect_uri")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| Error::InvalidAuthorizationUrl("no redirect_uri in URL".to_string()))?
            .parse()
            .map_err(|e: url::ParseError| Error::InvalidAuthorizationUrl(e.to_string()))?;

        let identity_provider: Option<String> = authorization_url
            .query_pairs()
            .find(|(k, _)| k == "identity_provider")
            .map(|(_, v)| v.to_string());

        if redirect_uri.scheme() != "http" {
            return Err(Error::InvalidAuthorizationUrl(
                "redirect_uri must use the `http` scheme".to_string(),
            ));
        }

        match redirect_uri.host() {
            Some(url::Host::Domain("localhost")) => {}
            Some(_) => {
                return Err(Error::InvalidAuthorizationUrl(
                    "redirect_uri must use the `localhost` host".to_string(),
                ))
            }
            None => {
                return Err(Error::InvalidAuthorizationUrl(
                    "redirect_uri must have a host".to_string(),
                ))
            }
        };

        // If there is no explicit port, assume the default port for the scheme.
        let port = redirect_uri.port().unwrap_or(80);

        let client = hyper::Client::builder()
            .build::<_, hyper::Body>(hyper_rustls::HttpsConnector::with_native_roots());

        let semaphore = Semaphore::new(1);

        Ok(Self {
            domain_name,
            region,
            client_id,
            scopes,
            port,
            identity_provider,
            client,
            semaphore,
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
    /// use url::Url;
    ///
    /// # fn main() {
    /// let url = Url::parse("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http://localhost:5001/&identity_provider=Azure").unwrap();
    /// let auth = AwsCognitoClientAuthenticator::from_authorization_url(&url).unwrap();
    ///
    /// assert_eq!(
    ///     auth.get_authorization_url().as_str(),
    ///     "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http%3A%2F%2Flocalhost%3A5001%2F&identity_provider=Azure",
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
    /// use url::Url;
    ///
    /// # fn main() {
    /// let url = Url::parse("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http://localhost:5001/&identity_provider=Azure").unwrap();
    /// let auth = AwsCognitoClientAuthenticator::from_authorization_url(&url).unwrap();
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
    /// use url::Url;
    ///
    /// # fn main() {
    /// let url = Url::parse("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http://localhost:5001/&identity_provider=Azure").unwrap();
    /// let auth = AwsCognitoClientAuthenticator::from_authorization_url(&url).unwrap();
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
    /// use url::Url;
    ///
    /// # fn main() {
    /// let url = Url::parse("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http://localhost:5001/&identity_provider=Azure").unwrap();
    /// let auth = AwsCognitoClientAuthenticator::from_authorization_url(&url).unwrap();
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

    async fn receive_authorization_code(&self) -> Result<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        let make_service = make_service_fn(move |socket: &AddrStream| {
            let tx = Arc::clone(&tx);
            debug!("new connection from: {}", socket.remote_addr());

            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let tx = Arc::clone(&tx);

                    async move {
                        debug!("received callback request: {:?}", req);

                        // We only accept calls to the root path.
                        if req.uri().path() != "/" {
                            warn!(
                                "rejecting request on unsupported path: {}",
                                req.uri().path()
                            );

                            return Ok::<_, hyper::Error>(
                                Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body(Body::empty())
                                    .unwrap(),
                            );
                        }

                        // Only GETs are valid.
                        if req.method() != hyper::Method::GET {
                            warn!(
                                "rejecting request with not allowed method: {}",
                                req.method(),
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::METHOD_NOT_ALLOWED)
                                .body(Body::empty())
                                .unwrap());
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
                                .body(Body::empty())
                                .unwrap());
                        };

                        if let Some(tx) = tx.lock().await.take() {
                            let _err = tx.send(code);

                            info!("authentication succeeded");

                            Ok(Response::new(Body::from(include_str!(
                                "static/authentication_succeeded.html"
                            ))))
                        } else {
                            warn!("ignoring second successful authentication call");

                            Ok(Response::builder()
                                .status(StatusCode::PRECONDITION_FAILED)
                                .body(Body::empty())
                                .unwrap())
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
            Err(Error::InternalServerError(e))
        } else {
            Ok(code)
        }
    }

    async fn receive_logout_confirmation(&self) -> Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        let make_service = make_service_fn(move |socket: &AddrStream| {
            let tx = Arc::clone(&tx);
            debug!("new connection from: {}", socket.remote_addr());

            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let tx = Arc::clone(&tx);

                    async move {
                        debug!("received callback request: {:?}", req);

                        // We only accept calls to the root path.
                        if req.uri().path() != "/" {
                            warn!(
                                "rejecting request on unsupported path: {}",
                                req.uri().path()
                            );

                            return Ok::<_, Infallible>(
                                Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body(Body::empty())
                                    .unwrap(),
                            );
                        }

                        // Only GETs are valid.
                        if req.method() != hyper::Method::GET {
                            warn!(
                                "rejecting request with not allowed method: {}",
                                req.method(),
                            );

                            return Ok(Response::builder()
                                .status(StatusCode::METHOD_NOT_ALLOWED)
                                .body(Body::empty())
                                .unwrap());
                        }

                        if let Some(tx) = tx.lock().await.take() {
                            let _err = tx.send(());

                            info!("logout succeeded");

                            Ok(Response::new(Body::from(include_str!(
                                "static/logout_succeeded.html"
                            ))))
                        } else {
                            warn!("ignoring second successful logout call");

                            Ok(Response::builder()
                                .status(StatusCode::PRECONDITION_FAILED)
                                .body(Body::empty())
                                .unwrap())
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
            Err(Error::InternalServerError(e))
        } else {
            Ok(())
        }
    }

    /// Get the authorization code by opening an interactive browser window.
    async fn get_authorization_code_interactive(&self) -> Result<String> {
        let authorization_url = self.get_authorization_url();

        info!("Opening web-browser at: {}", authorization_url);

        webbrowser::open(authorization_url.as_str()).map_err(Error::InteractiveProcessError)?;

        self.receive_authorization_code().await
    }

    /// Get a token set from an authorization code.
    async fn get_token_set_from_authorization_code(&self, code: &str) -> Result<ClientTokenSet> {
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(self.get_access_token_url().as_str())
            .header(
                hyper::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );

        let req = req
            .body(Body::from(format!(
                "grant_type=authorization_code&client_id={}&code={}&redirect_uri={}",
                self.client_id,
                code,
                self.get_redirect_uri(),
            )))
            .unwrap();

        let resp =
            self.client.request(req).await.map_err(|e| {
                Error::InternalError(format!("failed to execute HTTP request: {}", e))
            })?;
        let bytes = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(|e| Error::InternalError(format!("failed to read HTTP response: {}", e)))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| Error::InternalError(format!("failed to parse JSON token set: {}", e)))
    }

    /// Get user information from an access token.
    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfo> {
        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(self.get_user_info_url().as_str())
            .header(
                hyper::header::AUTHORIZATION,
                format!("Bearer {}", access_token),
            )
            .body(Body::empty())
            .unwrap();

        let resp =
            self.client.request(req).await.map_err(|e| {
                Error::InternalError(format!("failed to execute HTTP request: {}", e))
            })?;
        let bytes = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(|e| Error::InternalError(format!("failed to read HTTP response: {}", e)))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| Error::InternalError(format!("failed to parse JSON user info: {}", e)))
    }

    async fn lock(&self) -> Result<SemaphorePermit<'_>> {
        self.semaphore
            .acquire()
            .await
            .map_err(|e| Error::InternalError(format!("failed to acquire semaphore: {}", e)))
    }
}

#[async_trait]
impl Authenticator for AwsCognitoClientAuthenticator {
    /// Get a token set by opening a possible interactive browser window.
    async fn login(&self) -> Result<ClientTokenSet> {
        let _permit = self.lock().await?;

        let code = self.get_authorization_code_interactive().await?;

        self.get_token_set_from_authorization_code(&code).await
    }

    /// Get a token set from a refresh token.
    ///
    /// If the call does not return a new refresh token within the `TokenSet`, the specified
    /// refresh token will be filled in instead.
    async fn refresh_login(&self, refresh_token: &str) -> Result<ClientTokenSet> {
        let _permit = self.lock().await?;

        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(self.get_access_token_url().as_str())
            .header(
                hyper::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );

        let req = req
            .body(Body::from(format!(
                "grant_type=refresh_token&client_id={}&refresh_token={}",
                self.client_id, refresh_token,
            )))
            .unwrap();

        let resp =
            self.client.request(req).await.map_err(|e| {
                Error::InternalError(format!("failed to execute HTTP request: {}", e))
            })?;
        let bytes = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(|e| Error::InternalError(format!("failed to read HTTP response: {}", e)))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| Error::InternalError(format!("failed to parse JSON token set: {}", e)))
            .map(|mut token_set: ClientTokenSet| {
                if token_set.refresh_token.is_none() {
                    token_set.refresh_token = Some(refresh_token.to_owned());
                }

                token_set
            })
    }

    /// Logout by opening an interactive browser window.
    async fn logout(&self) -> Result<()> {
        let _permit = self.lock().await?;

        let logout_url = self.get_logout_url();

        info!("Opening web-browser at: {}", logout_url);

        webbrowser::open(logout_url.as_str()).map_err(Error::InteractiveProcessError)?;

        self.receive_logout_confirmation().await
    }
}
