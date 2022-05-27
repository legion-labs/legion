#[derive(Debug, Default)]
pub struct Context {
    // Initial headers and extensions provided by the client.
    request_headers: Option<http::HeaderMap>,
    request_extensions: Option<http::Extensions>,

    // Initial headers and extensions provided by the server.
    response_headers: Option<http::HeaderMap>,
    response_extensions: Option<http::Extensions>,

    // The full http request parts usable by the server.
    request: Option<http::request::Parts>,
    // The full http response parts usable by the client.
    response: Option<http::response::Parts>,
}

impl Context {
    pub fn request(&self) -> Option<&http::request::Parts> {
        self.request.as_ref()
    }

    pub fn set_request(&mut self, request: http::request::Parts) {
        self.request = Some(request);
    }

    pub fn response(&self) -> Option<&http::response::Parts> {
        self.response.as_ref()
    }

    pub fn set_response(&mut self, response: http::response::Parts) {
        self.response = Some(response);
    }

    pub fn request_headers(&mut self) -> Option<http::HeaderMap> {
        self.request_headers.take()
    }

    pub fn set_request_headers(&mut self, headers: http::HeaderMap) {
        self.request_headers = Some(headers);
    }

    pub fn request_extensions(&mut self) -> Option<http::Extensions> {
        self.request_extensions.take()
    }

    pub fn set_request_extensions(&mut self, extensions: http::Extensions) {
        self.request_extensions = Some(extensions);
    }

    pub fn response_headers(&mut self) -> Option<http::HeaderMap> {
        self.response_headers.take()
    }

    pub fn set_response_headers(&mut self, headers: http::HeaderMap) {
        self.response_headers = Some(headers);
    }

    pub fn response_extensions(&mut self) -> Option<http::Extensions> {
        self.response_extensions.take()
    }

    pub fn set_response_extensions(&mut self, extensions: http::Extensions) {
        self.response_extensions = Some(extensions);
    }
}
