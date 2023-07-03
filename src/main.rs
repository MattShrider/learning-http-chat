use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use learning_http_chat::*;

/// If this process has this many threads alive at once, do not accept
/// any more connections.
const MAX_THREADS: usize = 1000;
const ADDRESS: &str = "127.0.0.1:8080";
fn main() {
    let listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Listening to {}", ADDRESS);

    let thread_count = Arc::new(Mutex::new(0));
    for stream_result in listener.incoming() {
        let thread_count = thread_count.clone();
        //Drop the Mutex asap
        {
            if *thread_count.lock().unwrap() > MAX_THREADS {
                eprintln!("Forced to skip connection due to max threads reached");
                continue;
            }
        }

        if let Ok(stream) = stream_result {
            // This implicitly detaches the thread. We're relying on
            // handle_connection to set timeouts on reading from the stream
            // to avoid resource leaks.
            thread::spawn(move || {
                {
                    let mut count = thread_count.lock().unwrap();
                    *count += 1;
                    println!("Thread Started: {}", *count);
                }
                let result = handle_connection(stream);
                {
                    let mut count = thread_count.lock().unwrap();
                    *count -= 1;
                    println!("Thread Closed: {}", *count);
                }
                result
            });
        } else {
            eprintln!("{:#?}", stream_result.err());
        }
    }
}

const HTTP_VERSION: &str = HttpVersion::Http1_1.as_str();
const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_secs(100));
fn handle_connection(mut stream: TcpStream) -> Result<(), HttpRequestValidationErr> {
    stream.set_read_timeout(STREAM_TIMEOUT).unwrap();
    stream.set_write_timeout(STREAM_TIMEOUT).unwrap();
    println!("Connection Established with {:?}", stream);

    let request = learning_http_chat::HttpRequest::from_stream(&stream);
    println!("REQUEST: {:#?}", request);

    let response = match request {
        Ok(HttpRequest {
            method: _,
            resource: _,
            headers: _,
            body: Some(req_body),
            version: _,
        }) => prepare_response(200, "ok", Some(&req_body)),
        Ok(_) => prepare_response(200, "ok", None),
        Err(_) => {
            // todo!("Match the error type here to provide better feedback");
            prepare_response(
                400,
                "request_error",
                Some("I have no idea what you just sent"),
            )
        }
    };

    stream.write_all(response.as_bytes()).unwrap();
    println!("RESPONSE: \n{:#?}", response);

    Ok(())
}

const HTTP_DELIMIT: &str = "\r\n";
fn prepare_response(status: u32, status_text: &str, body: Option<&str>) -> String {
    let status_line = format!("{HTTP_VERSION} {status} {status_text}{HTTP_DELIMIT}");
    let mut headers_block: String = String::from("");
    let mut body_block: String = String::from("");
    if let Some(body) = body {
        headers_block = format!("Content-Length: {}{HTTP_DELIMIT}", body.len());
        body_block = format!("{HTTP_DELIMIT}{body}");
    }
    format!("{status_line}{headers_block}{body_block}")
}
