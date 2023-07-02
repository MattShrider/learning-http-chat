use std::borrow::BorrowMut;
use std::io::{BufRead, BufReader, Write};
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

use chat::HttpRequestValidationErr;

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

    let response = "HTTP/1.1 200 OK\r\n\r\nHowdy\r\n";
    stream.write_all(response.as_bytes()).unwrap();
    println!("RESPONSE: {:#?}", response);

    Ok(())
}
