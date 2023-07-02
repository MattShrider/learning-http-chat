use std::borrow::BorrowMut;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
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

    let mut errs = vec![];

    let stream_reader = BufReader::new(&stream);
    let mut lines_iter = stream_reader.lines().filter_map(|line| match line {
        Ok(v) => Some(v),
        Err(why) => {
            errs.push(why);
            None
        }
    });

    let request = chat::HttpRequest::from_lines(lines_iter.borrow_mut());
    println!("{:#?}", request);

    let response = "HTTP/1.1 200 OK\r\n\r\nHowdy\r\n";
    stream.write_all(response.as_bytes()).unwrap();

    Ok(())
}
