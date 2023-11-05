mod utils;
use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use utils::Response;

use crate::utils::{Request, ResponseHeader};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_connection(&stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: &TcpStream) {
    let buf_reader = BufReader::new(stream);

    match Request::try_from(buf_reader) {
        Ok(request) => {
            let response = Response::new(request);
            stream.write(response.format_response().as_bytes()).unwrap();
        }
        Err(e) => {
            eprintln!("Failed reading the request: {:?}", e);
            stream
                .write_all(ResponseHeader::get(ResponseHeader::HttpBad).as_bytes())
                .unwrap();
        }
    };
}
