use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use http::{Request, Response, StatusCode};
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Server,
};
use lgn_tracing::{debug, info, warn};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreUserInfoClaims},
    reqwest::async_http_client,
    AccessToken, AccessTokenHash, AuthType, AuthorizationCode, ClientId, CsrfToken, IssuerUrl,
    Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope,
    SubjectIdentifier, TokenResponse,
};
use tokio::sync::{oneshot, Mutex};
use url::Url;

use super::{Authenticator, ClientTokenSet, Error, Result, UserInfo};

const DEFAULT_PORT: u16 = 3000;

pub struct OAuthClient {
    client: CoreClient,
    client_id: ClientId,
    provider_metadata: CoreProviderMetadata,
    redirect_uri: Option<RedirectUrl>,
    // A port is identified for each transport protocol and
    // address combination by a 16-bit unsigned number, known as the port number
    // https://en.wikipedia.org/wiki/Port_(computer_networking)
    port: u16,
}

impl OAuthClient {
    pub async fn new<'a, IU, ID>(issuer_url: IU, client_id: ID) -> Result<Self>
    where
        IU: Into<String>,
        ID: Into<String>,
    {
        let issuer_url = IssuerUrl::new(issuer_url.into())
            .map_err(|error| Error::Internal(format!("{}", error)))?;

        let client_id = ClientId::new(client_id.into());

        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .map_err(|error| Error::Internal(format!("{}", error)))?;

        let client =
            CoreClient::from_provider_metadata(provider_metadata.clone(), client_id.clone(), None)
                .set_auth_type(AuthType::RequestBody);

        Ok(Self {
            client,
            client_id,
            port: DEFAULT_PORT,
            provider_metadata,
            redirect_uri: None,
        })
    }

    pub fn set_redirect_uri(mut self, redirect_uri: &Url) -> Result<Self> {
        let redirect_uri = RedirectUrl::new(redirect_uri.clone().into())
            .map_err(|error| Error::Internal(format!("{}", error)))?;

        self.client = self.client.set_redirect_uri(redirect_uri.clone());

        self.redirect_uri = Some(redirect_uri);

        Ok(self)
    }

    pub fn set_port<RU>(&mut self, port: u16) -> &mut Self {
        self.port = port;

        self
    }

    pub async fn user_info(
        &self,
        access_token: AccessToken,
        subject: Option<SubjectIdentifier>,
    ) -> Result<CoreUserInfoClaims> {
        self.client
            .user_info(access_token, subject)
            .map_err(Error::internal)?
            .request_async(async_http_client)
            .await
            .map_err(Error::internal)
    }

    // TODO: Make this more generic
    /// Partly implements <https://openid.net/specs/openid-connect-rpinitiated-1_0.html>
    /// See <https://github.com/ramosbugs/openidconnect-rs/issues/40> for more
    /// Notice that not all oauth providers follow the rfc, Cognito for example
    /// use `"redirect_uri"` instead of `"post_logout_redirect_uri"`
    fn logout_url(&self) -> String {
        // Authorization url is required as per the rfc and will always be present,
        // so we can use it as a base url
        let mut url = self
            .provider_metadata
            .authorization_endpoint()
            .url()
            .clone();

        url = url.join("/logout").unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();

            query_pairs.append_pair("client_id", &self.client_id);

            if let Some(ref redirect_uri) = self.redirect_uri {
                query_pairs.append_pair("redirect_uri", redirect_uri);
            }
        }

        url.to_string()
    }

    /// Get user information from an access token.
    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfo> {
        let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http2()
            .build();

        let client = hyper::Client::builder().build::<_, hyper::Body>(https_connector);

        let url = self.provider_metadata.userinfo_endpoint().ok_or_else(|| {
            Error::Internal("userinfo endpoint not provided by the issuer".into())
        })?;

        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(url.as_str())
            .header(
                hyper::header::AUTHORIZATION,
                format!("Bearer {}", access_token),
            )
            .body(Body::empty())
            .unwrap();

        let resp = client
            .request(req)
            .await
            .map_err(|e| Error::Internal(format!("failed to execute HTTP request: {}", e)))?;

        let bytes = hyper::body::to_bytes(resp.into_body())
            .await
            .map_err(|e| Error::Internal(format!("failed to read HTTP response: {}", e)))?;

        serde_json::from_slice(&bytes)
            .map_err(|e| Error::Internal(format!("failed to parse JSON user info: {}", e)))
    }

    async fn receive_authorization_code(&self) -> Result<(String, String)> {
        // [`make_service_fn`] and [`make_service`] both expect an [`FnMut`] closure,
        // and since we're in an `async` context we can't just move the tokio channels
        // and need to use an [`Arc`].
        // Since we need to clone the channels twice the [`Arc::try_unwrap`] + [`Mutext::into_inner`]
        // trick won't work...
        // Our remaining solution is the [`Option::take`] trick
        // C.f. https://stackoverflow.com/questions/29177449/how-to-take-ownership-of-t-from-arcmutext

        let (tx_params, rx_params) = oneshot::channel();
        let tx_params = Arc::new(Mutex::new(Some(tx_params)));

        let (tx_done, rx_done) = oneshot::channel();
        let tx_done = Arc::new(Mutex::new(Some(tx_done)));

        let make_service = make_service_fn(move |socket: &AddrStream| {
            let tx_params = Arc::clone(&tx_params);
            let tx_done = Arc::clone(&tx_done);

            debug!("new connection from: {}", socket.remote_addr());

            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let tx_params = Arc::clone(&tx_params);
                    let tx_done = Arc::clone(&tx_done);

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
                        let query = req
                            .uri()
                            .query()
                            .map(|v| url::form_urlencoded::parse(v.as_bytes()).into_owned());

                        let mut query = if let Some(query) = query {
                            query
                        } else {
                            warn!("rejecting request as query string couldn't be parsed");

                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::empty())
                                .unwrap());
                        };

                        let code = if let Some(code) =
                            query.find_map(|(k, v)| if k == "code" { Some(v) } else { None })
                        {
                            code
                        } else {
                            warn!("rejecting request without code");

                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::empty())
                                .unwrap());
                        };

                        let state = if let Some(state) =
                            query.find_map(|(k, v)| if k == "state" { Some(v) } else { None })
                        {
                            state
                        } else {
                            warn!("rejecting request without state");

                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::empty())
                                .unwrap());
                        };

                        let tx_params = tx_params.lock().await.take().unwrap();
                        let tx_done = tx_done.lock().await.take().unwrap();

                        let _error = tx_params.send((code, state));
                        let _error = tx_done.send(());

                        info!("authentication succeeded");

                        Ok(Response::new(Body::from(include_str!(
                            "static/authentication_succeeded.html"
                        ))))
                    }
                }))
            }
        });

        let server = Server::bind(&self.get_callback_addr())
            .serve(make_service)
            .with_graceful_shutdown(async {
                rx_done.await.unwrap();

                info!("temporary http server shutdown");
            });

        if let Err(error) = server.await {
            Err(Error::InternalServer(error))
        } else {
            let code_and_state = rx_params.await.unwrap();

            info!("received authorization code and state : shutting down temporary HTTP server");

            Ok(code_and_state)
        }
    }

    async fn receive_logout_confirmation(&self) -> Result<()> {
        // [`make_service_fn`] and [`make_service`] both expect an [`FnMut`] closure,
        // and since we're in an `async` context we can't just move the tokio channels
        // and need to use an [`Arc`].
        // Since we need to clone the channels twice the [`Arc::try_unwrap`] + [`Mutext::into_inner`]
        // trick won't work...
        // Our remaining solution is the [`Option::take`] trick
        // C.f. https://stackoverflow.com/questions/29177449/how-to-take-ownership-of-t-from-arcmutext

        let (tx_done, rx_done) = oneshot::channel();
        let tx_done = Arc::new(Mutex::new(Some(tx_done)));

        let make_service = make_service_fn(move |socket: &AddrStream| {
            let tx_done = Arc::clone(&tx_done);

            debug!("new connection from: {}", socket.remote_addr());

            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let tx_done = Arc::clone(&tx_done);

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

                        let tx_done = tx_done.lock().await.take().unwrap();

                        let _error = tx_done.send(());

                        info!("logout succeeded");

                        Ok(Response::new(Body::from(include_str!(
                            "static/logout_succeeded.html"
                        ))))
                    }
                }))
            }
        });

        let server = Server::bind(&self.get_callback_addr())
            .serve(make_service)
            .with_graceful_shutdown(async {
                rx_done.await.unwrap();

                info!("temporary http server shutdown");
            });

        if let Err(error) = server.await {
            Err(Error::InternalServer(error))
        } else {
            info!("received logout confirmation: shutting down temporary HTTP server");

            Ok(())
        }
    }

    fn get_callback_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.port))
    }
}

#[async_trait]
impl Authenticator for OAuthClient {
    async fn login(
        &self,
        scopes: &[String],
        extra_params: &Option<HashMap<String, String>>,
    ) -> Result<ClientTokenSet> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut authorization_url = self.client.authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        for scope in scopes {
            authorization_url = authorization_url.add_scope(Scope::new(scope.into()));
        }

        if let Some(extra_params) = extra_params {
            for (name, value) in extra_params {
                authorization_url = authorization_url.add_extra_param(&*name, &*value);
            }
        }

        let (authorization_url, csrf_token, nonce) =
            authorization_url.set_pkce_challenge(pkce_challenge).url();

        info!("Opening web-browser at: {}", authorization_url);

        webbrowser::open(authorization_url.as_str()).map_err(Error::InteractiveProcess)?;

        let (code, state) = self.receive_authorization_code().await?;

        if state != *csrf_token.secret() {
            return Err(Error::Internal(
                "Received csrf code and expected csrf code don't match".into(),
            ));
        }

        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(Error::internal)?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| Error::Internal("Server did not return an ID token".into()))?;

        let claims = id_token
            .claims(&self.client.id_token_verifier(), &nonce)
            .map_err(Error::internal)?;

        if let Some(expected_access_token_hash) = claims.access_token_hash() {
            let actual_access_token_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                &id_token.signing_alg().map_err(Error::internal)?,
            )
            .map_err(Error::internal)?;

            if actual_access_token_hash != *expected_access_token_hash {
                return Err(Error::Internal(
                    "Received access token hash and expected access token hash don't match".into(),
                ));
            }
        }

        token_response.try_into()
    }

    /// Get a token set from a refresh token.
    ///
    /// If the call does not return a new refresh token within the `TokenSet`,
    /// the specified refresh token will be filled in instead.
    async fn refresh_login(&self, refresh_token: String) -> Result<ClientTokenSet> {
        let token_response = self
            .client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(Error::internal)?;

        token_response.try_into()
    }

    /// Logout by opening an interactive browser window
    async fn logout(&self) -> Result<()> {
        let logout_url = self.logout_url();

        info!("Opening web-browser at: {}", logout_url);

        webbrowser::open(logout_url.as_str()).map_err(Error::InteractiveProcess)?;

        self.receive_logout_confirmation().await
    }
}
