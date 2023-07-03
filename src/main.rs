use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

use learning_http_chat::*;

const ADDRESS: &str = "127.0.0.1:8080";
fn main() {
    let (tx, rx) = mpsc::channel();
    let listener = TcpListener::bind(ADDRESS).unwrap();
    println!("Listening to {}", ADDRESS);
    let handler_thread = thread::spawn(move || {
        for stream in rx {
            match handle_connection(stream) {
                Err(errs) => eprintln!("connection error {:?}", errs),
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
