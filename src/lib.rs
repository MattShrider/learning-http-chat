use std::{borrow::BorrowMut, collections::HashMap};

#[derive(Debug)]
pub enum HttpRequestValidationErr {
    Headline,
    MethodMissing,
    MethodMalformed,
    ResourceMissing,
    ResourceMalformed,
    HttpVersionMissing,
    HttpVersionMalformed,
    HeadersMalformed,
    BodyMalformed,
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    PUT,
    PATCH,
    POST,
    DELETE,
}

impl HttpMethod {
    pub fn parse(method_str: &str) -> Result<Self, HttpRequestValidationErr> {
        match method_str.to_uppercase().as_str() {
            "GET" => Ok(Self::GET),
            "PUT" => Ok(Self::PUT),
            "POST" => Ok(Self::POST),
            "PATCH" => Ok(Self::PATCH),
            "DELETE" => Ok(Self::DELETE),
            _ => Err(HttpRequestValidationErr::MethodMalformed),
        }
    }
}

#[derive(Debug)]
enum HttpVersion {
    Http1_1,
    Http2,
}

impl HttpVersion {
    fn parse(word: &str) -> Result<Self, HttpRequestValidationErr> {
        match word.to_uppercase().as_str() {
            "HTTP/1.1" => Ok(HttpVersion::Http1_1),
            "HTTP/2" => Ok(HttpVersion::Http2),
            _ => Err(HttpRequestValidationErr::HttpVersionMalformed),
        }
    }
}

#[derive(Debug)]
struct HttpMethodSection {
    method: HttpMethod,
    resource: String,
    version: HttpVersion,
}

impl HttpMethodSection {
    fn parse_lines<T>(lines: &mut T) -> Result<Self, HttpRequestValidationErr>
    where
        T: Iterator<Item = String>,
    {
        let head: String = lines.next().ok_or(HttpRequestValidationErr::Headline)?;
        let mut head_iter = head.split_whitespace();

        let method = head_iter
            .next()
            .map(HttpMethod::parse)
            .ok_or(HttpRequestValidationErr::MethodMissing)??;
        let resource = head_iter
            .next()
            .map(|slc| slc.to_string())
            .ok_or(HttpRequestValidationErr::ResourceMissing)?;
        let version = head_iter
            .next()
            .map(HttpVersion::parse)
            .ok_or(HttpRequestValidationErr::HttpVersionMissing)??;

        Ok(HttpMethodSection {
            method,
            resource,
            version,
        })
    }
}

#[derive(Debug)]
pub struct HttpHeaders(HashMap<String, Vec<String>>);

impl HttpHeaders {
    fn parse_lines<T>(lines_iter: &mut T) -> Result<Self, HttpRequestValidationErr>
    where
        T: Iterator<Item = String>,
    {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for line in lines_iter {
            if line.is_empty() {
                break;
            }
            let mut words = line.split(':').map(|w| w.trim().to_owned());
            let key = words
                .next()
                .ok_or(HttpRequestValidationErr::HeadersMalformed)?;
            let value = words
                .next()
                .ok_or(HttpRequestValidationErr::HeadersMalformed)?;

            if let Some(list) = map.get_mut(&key) {
                list.push(value);
            } else {
                map.insert(key, vec![value]);
            }
        }

        Ok(HttpHeaders(map))
    }
}

#[derive(Debug)]
struct HttpBody(Option<String>);

impl HttpBody {
    fn parse_lines<T>(lines: &mut T) -> Vec<String>
    where
        T: Iterator<Item = String>,
    {
        lines.take_while(|line| !line.is_empty()).collect()
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    resource: String,
    headers: HttpHeaders,
    body: Vec<String>,
    version: HttpVersion,
}

impl HttpRequest {
    pub fn from_lines<T>(lines_iter: &mut T) -> Result<Self, HttpRequestValidationErr>
    where
        T: Iterator<Item = String>,
    {
        let HttpMethodSection {
            method,
            version,
            resource,
        } = HttpMethodSection::parse_lines(lines_iter)?;
        let headers = HttpHeaders::parse_lines(lines_iter)?;
        let body = HttpBody::parse_lines(lines_iter);

        Ok(HttpRequest {
            method,
            version,
            resource,
            headers,
            body,
        })
    }
}
