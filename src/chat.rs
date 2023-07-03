use crate::{HttpMethod, HttpPath, HttpRequest};

struct HttpHandler {
    method: HttpMethod,
    path: HttpPath,
    handler: Box<dyn Fn(HttpRequest)>,
}

pub struct HttpRouter {
    handlers: Vec<HttpHandler>,
}

impl HttpRouter {
    pub fn attach<F>(&mut self, method: HttpMethod, path: HttpPath, handler: F)
    where
        F: Fn(HttpRequest) + 'static,
    {
        self.handlers.push(HttpHandler {
            method,
            path,
            handler: Box::new(handler),
        })
    }
}
