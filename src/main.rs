use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::{net::TcpListener, thread};

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

use chat::*;

fn main() {
    let (tx, rx) = mpsc::channel();
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
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

const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_millis(100));
fn handle_connection(mut stream: TcpStream) -> Result<(), HttpRequestValidationErr> {
    stream.set_read_timeout(STREAM_TIMEOUT).unwrap();
    stream.set_write_timeout(STREAM_TIMEOUT).unwrap();
    println!("Connection Established with {:?}", stream);

    let request = chat::HttpRequest::from_stream(&stream);
    println!("REQUEST: {:#?}", request);

    let response = match request {
        Ok(HttpRequest {
            method: _,
            resource: _,
            headers: _,
            body: Some(req_body),
            version: _,
        }) => {
            format!("HTTP/1.1 200 ok\r\n\r\n{}\r\n", req_body)
        }
        Ok(_) => {
            let message = "<No body provided>";
            format!("HTTP/1.1 200 ok\r\n\r\n{}\r\n", message)
        }
        Err(_) => {
            // todo!("Match the error type here to provide better feedback");
            let message = "No idea what you just sent my man";
            format!("HTTP/1.1 400 request_malformed\r\n\r\n{}\r\n", message)
        }
    };

    stream.write_all(response.as_bytes()).unwrap();
    println!("RESPONSE: {:#?}", response);

    Ok(())
}
