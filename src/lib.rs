use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    mem::size_of,
    net::TcpStream,
};

/// Default capacity when reading lines
const BUF_CAPACTIY: usize = 1000 * size_of::<char>();

#[derive(Debug)]
pub enum HttpRequestValidationErr {
    Headline(String),
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

impl TryFrom<&str> for HttpMethod {
    type Error = HttpRequestValidationErr;

    fn try_from(method_str: &str) -> Result<Self, Self::Error> {
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
pub enum HttpVersion {
    Http1_1,
    Http2,
}

impl HttpVersion {
    pub const fn as_str(&self) -> &str {
        match self {
            HttpVersion::Http1_1 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
        }
    }
}

impl ToString for HttpVersion {
    fn to_string(&self) -> String {
        self.as_str().to_owned()
    }
}

impl TryFrom<&str> for HttpVersion {
    type Error = HttpRequestValidationErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
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
    fn parse_lines<T: BufRead>(reader: &mut T) -> Result<Self, HttpRequestValidationErr> {
        let mut buffer = String::with_capacity(BUF_CAPACTIY);
        let bytes_read = reader
            .read_line(&mut buffer)
            .map_err(|e| HttpRequestValidationErr::Headline(e.to_string()))?;

        if bytes_read == 0 {
            return Err(HttpRequestValidationErr::Headline(
                "No bytes read in headline".to_owned(),
            ));
        }

        let mut head_iter = buffer.split_whitespace();
        let method = head_iter
            .next()
            .map(HttpMethod::try_from)
            .ok_or(HttpRequestValidationErr::MethodMissing)??;
        let resource = head_iter
            .next()
            .map(|slc| slc.to_string())
            .ok_or(HttpRequestValidationErr::ResourceMissing)?;
        let version = head_iter
            .next()
            .map(HttpVersion::try_from)
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
    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.0.get(&key.to_string().to_lowercase())
    }

    fn parse_lines<T: BufRead>(reader: &mut T) -> Result<Self, HttpRequestValidationErr> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();

        let mut buffer = String::with_capacity(BUF_CAPACTIY);
        loop {
            buffer.clear();
            let bytes_read = reader
                .read_line(&mut buffer)
                .map_err(|_| HttpRequestValidationErr::HeadersMalformed)?;
            let line = buffer.trim();
            if bytes_read == 0 || line.is_empty() {
                break;
            }

            let (key, value) = line
                .split_once(':')
                .map(|(left, right)| (left.trim().to_lowercase(), right.trim().to_owned()))
                .ok_or(HttpRequestValidationErr::HeadersMalformed)?;

            // .map(|w| w.trim().to_owned().to_lowercase());

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

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub resource: String,
    pub headers: HttpHeaders,
    pub body: Option<String>,
    pub version: HttpVersion,
}

// Completely arbitrary DOS protection
const MAX_LINE_LENGTH: usize = size_of::<char>() * 80_000;

impl HttpRequest {
    pub fn from_stream(stream: &TcpStream) -> Result<Self, HttpRequestValidationErr> {
        let mut stream_reader = BufReader::new(stream).take(MAX_LINE_LENGTH.try_into().unwrap());

        let HttpMethodSection {
            method,
            version,
            resource,
        } = HttpMethodSection::parse_lines(&mut stream_reader)?;

        let headers = HttpHeaders::parse_lines(&mut stream_reader)?;

        let body = match headers.get("content-length") {
            Some(vec) => {
                let content_length = vec
                    .get(0)
                    .unwrap()
                    .parse::<usize>()
                    .map_err(|_| HttpRequestValidationErr::HeadersMalformed)?;

                let mut buf = vec![0; content_length];
                stream_reader
                    .read_exact(&mut buf)
                    .map_err(|_| HttpRequestValidationErr::BodyMalformed)?;
                let body =
                    String::from_utf8(buf).map_err(|_| HttpRequestValidationErr::BodyMalformed)?;
                Ok(if body.is_empty() { None } else { Some(body) })
            }
            _ => Ok(None),
        }?;

        Ok(HttpRequest {
            method,
            version,
            resource,
            headers,
            body,
        })
    }
}
