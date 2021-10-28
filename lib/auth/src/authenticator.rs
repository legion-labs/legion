use std::{net::SocketAddr, sync::Arc};

use anyhow::bail;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use log::{debug, info, warn};
use tokio::sync::Mutex;
use url::Url;

pub struct Authenticator {
    pub domain_name: String,
    pub region: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub port: u16,
}

impl Authenticator {
    /// Creates an authenticator from a valid AWS Cognito URL.
    ///
    /// # Example
    ///
    /// ```
    /// use legion_auth::authenticator::Authenticator;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=4a6vcgqr108in51n3di730hk25&response_type=code&scope=aws.cognito.signin.user.admin+email+openid&redirect_uri=http://localhost:5001/").unwrap();
    /// let auth = Authenticator::from_authorization_url(&url).unwrap();
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
        })
    }

    fn get_callback_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], self.port))
    }

    fn get_authorization_url(&self) -> String {
        let mut url = Url::parse(&format!(
            "https://{}.auth.{}.amazoncognito.com/oauth2/authorize",
            self.domain_name, self.region
        ))
        .unwrap();

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.scopes.join("+"))
            .append_pair("redirect_uri", &format!("http://localhost:{}/", self.port));

        url.to_string()
    }

    fn get_logout_url(&self) -> String {
        let mut url = Url::parse(&format!(
            "https://{}.auth.{}.amazoncognito.com/logout",
            self.domain_name, self.region
        ))
        .unwrap();

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &format!("http://localhost:{}/", self.port));

        url.to_string()
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
                info!("received authorization code `{}`: shutting down", code);
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
                info!("received logout confirmation: shutting down");
            });

        if let Err(e) = server.await {
            bail!("failed to serve callback server: {}", e);
        }

        Ok(())
    }

    pub async fn get_authorization_code(&self) -> anyhow::Result<String> {
        let authorization_url = self.get_authorization_url();

        info!("Opening web-browser at: {}", authorization_url);

        webbrowser::open(authorization_url.as_str())?;

        self.receive_authorization_code().await
    }

    pub async fn logout(&self) -> anyhow::Result<()> {
        let logout_url = self.get_logout_url();

        info!("Opening web-browser at: {}", logout_url);

        webbrowser::open(logout_url.as_str())?;

        self.receive_logout_confirmation().await
    }
}
