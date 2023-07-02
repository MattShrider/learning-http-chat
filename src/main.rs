use std::collections::HashMap;
use std::fmt::Error;
use std::io::{BufRead, BufReader, Read, Write, Lines};
use std::net::TcpStream;
use std::{io, net::TcpListener, thread};

// #[derive(Debug)]
// struct User {
//     id: String,
//     name: String
// }
//
// #[derive(Debug)]
// struct Message {
//     text: String,
// }

use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let handler_thread = thread::spawn(move || {
        for stream in rx {
            match handle_connection(stream) {
                Err(errs) => eprintln!("Connection Error {:?}", errs),
                _ => (),
            }
        }
    });

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => tx.send(stream).expect("Can't send to thread"),
            Err(why) => {
                eprintln!("{:?}", why);
            }
        }
    }

    drop(tx);
    handler_thread.join().unwrap();
}


fn handle_connection(mut stream: TcpStream) -> Result<(), Vec<std::io::Error>> {
    println!("Connection Established with {:?}", stream);

    let has_content = false;
    let mut errs = vec![];
    let stream_reader = BufReader::new(&stream);
    let request: Vec<_> = stream_reader
        .lines()
        .filter_map(|result| match result {
            Ok(v) => {
                if ()
                Some(v)
            },
            Err(why) => {
                errs.push(why);
                None
            }
        })
        .take_while(|line| !line.is_empty())
        .collect();

    if !errs.is_empty() {
        return Err(errs);
    }

    println!("Request {:#?}", request);

    let response = "HTTP/1.1 200 OK\r\n\r\nHowdy\r\n";
    stream.write_all(response.as_bytes()).unwrap();

    Ok(())
}

enum HttpRequestValidationErr {
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

enum HttpMethod {
    Unknown,
    GET,
    PUT,
    PATCH,
    POST,
    DELETE,
}

impl HttpMethod {
    fn from(method_str: &str) -> Self {
        match method_str.to_uppercase().as_str() {
            "GET" => Self::GET,
            "PUT" => Self::PUT,
            "POST" => Self::POST,
            "PATCH" => Self::PATCH,
            "DELETE" => Self::DELETE,
            _ => Self::Unknown
        }
    }
}

struct HttpHeaders(HashMap<String, Vec<String>>);

impl HttpHeaders {
    /// Creates a Headers struct from an iterator of the raw lines.
    ///
    /// The headers are constructed once an empty line is reached. This
    /// function mutates the iterator passed in, making it convenient to
    /// reuse the same Iterator for getting the Http Request body.
    fn from<'a, T>(lines_iter: &mut T) -> Result<Self, HttpRequestValidationErr>
        where T: Iterator<Item=&'a str>
    {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        lines_iter
            .take_while(|line| !line.is_empty())
            .map(|line| {
                let mut words = line.split_whitespace();
                let key = words.next()
                    .map(|s| s.get(..s.len()));
                let value = words.next();

                match (key, value) {
                    (Some(key), Some(value)) => {
                        let vec = map
                            .get(key)
                            .unwrap_or(&vec![])
                            .as_mut();
                        map.insert(key, value)
                        
                    },
                    _ => Err(HttpRequestValidationErr::HeadersMalformed)
                }
                
            });

        Ok(HttpHeaders(map))
    }
}

struct HttpRequest {
    method: HttpMethod,
    resource: String,
    headers: HashMap<String, Vec<String>>
}

impl HttpRequest {
    fn from_lines(lines: &Vec<&str>) -> Result<Self, HttpRequestValidationErr> {
        let mut lines_iter = lines.iter();
        let mut head_iter = lines_iter.next()
            .map_or(Err(HttpRequestValidationErr::Headline), |line| { Ok(line.split_whitespace()) })?;
        let method = head_iter.next().map(HttpMethod::from).ok_or(HttpRequestValidationErr::MethodMissing)?;
        let resource = head_iter.next().map(|slc| slc.to_string()).ok_or(HttpRequestValidationErr::ResourceMissing)?;
        let schema_version = head_iter.next().ok_or(HttpRequestValidationErr::HttpVersionMissing)?;

        Ok(HttpRequest {
            method,
            resource,
            headers: HashMap::new()
        })
    }
}

fn get_content_length(line: &str) -> Option<usize> {
    if (line.to_lowercase().contains("content-length:")) {
        return line.split_whitespace()
            .skip(1)
            .next()
            .map(|val| val.parse::<usize>().unwrap_or(None));
    }
    None
}

// fn handle_message(message: Message) {
//     println!("{:?}", message);
// }
