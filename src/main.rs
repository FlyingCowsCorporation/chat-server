mod http;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use http::HttpFormatter;
use http::HttpParser;
use http::HttpRequest;


fn main() {
    let listener = TcpListener::bind("0.0.0.0:15000").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let client_addr = stream.peer_addr().unwrap();

        println!("--> Connection established: {}", client_addr);

        handle_connection(stream);
    }
}

fn handle_connection(stream : TcpStream) {

    let mut connection = Connection::new(stream);
    let request_str = connection.read_data();
    
    let request = HttpParser::parse(&request_str);

    match request {
        Err(msg) => {
            println!("  ! Error: {}", msg);
        },
        Ok(HttpRequest::GET(data)) => {
            println!("    GET {}\n    Body: {}", data.location, data.body);
            let response = HttpFormatter::ok("GET ACCEPTED");
            connection.write_data(&response);
        },
        Ok(HttpRequest::POST(data)) => {
            println!("    POST {}\n    Body: {}", data.location, data.body);
            let response = HttpFormatter::ok("POST ACCEPTED");
            connection.write_data(&response);
        },
    }
    println!("  > Connection closed.");
}

struct Connection{
    stream : TcpStream,
}

impl Connection{

    fn new(stream : TcpStream) -> Connection {
        Connection{
            stream
        }
    }

    fn read_data(&mut self) -> String {
        let mut buffer = [0; 512];

        self.stream.read(&mut buffer).unwrap();

        let data = String::from_utf8_lossy(&buffer[..]);
        data.to_string()
    }

    fn write_data(&mut self, data : &str) {
        self.stream.write(data.as_bytes()).unwrap();
        self.stream.flush().unwrap();
    }
}