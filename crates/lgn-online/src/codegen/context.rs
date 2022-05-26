#[derive(Debug, Default)]
pub struct Context {
    request: Option<Request>,
    response: Option<Response>,
}

#[derive(Debug, Default)]
pub struct Request {
    pub method: http::Method,
    pub uri: http::Uri,
    pub version: http::Version,
    pub headers: http::HeaderMap,
    pub extensions: http::Extensions,
}

#[derive(Debug, Default)]
pub struct Response {
    pub headers: http::HeaderMap,
    pub extensions: http::Extensions,
}

impl Context {
    pub fn from(request: Option<Request>, response: Option<Response>) -> Self {
        Self { request, response }
    }

    pub fn request(&self) -> &Option<Request> {
        &self.request
    }

    pub fn set_request(&mut self, request: Request) {
        self.request = Some(request);
    }

    pub fn response(&self) -> &Option<Response> {
        &self.response
    }

    pub fn set_response(&mut self, response: Response) {
        self.response = Some(response);
    }
}

impl From<http::request::Parts> for Request {
    fn from(parts: http::request::Parts) -> Self {
        Self {
            method: parts.method,
            uri: parts.uri,
            version: parts.version,
            headers: parts.headers,
            extensions: parts.extensions,
        }
    }
}

impl From<http::response::Parts> for Response {
    fn from(parts: http::response::Parts) -> Self {
        Self {
            headers: parts.headers,
            extensions: parts.extensions,
        }
    }
}
