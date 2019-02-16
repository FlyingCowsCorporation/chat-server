mod http;

use std::io;
use std::io::prelude::*;

use std::net::TcpListener;
use std::net::TcpStream;

use http::HttpFormatter;
use http::HttpParser;
use http::HttpRequest;
use http::HttpRequestContent;

fn main() {
    let mut server = ChatServer::new();
    server.start();
}

struct ChatServer {
    waiting_connections : Vec<Connection>,
}

impl ChatServer {

    fn new() -> ChatServer {
        ChatServer {
            waiting_connections: Vec::new()
        }
     }

    fn start(&mut self){
        let listener = TcpListener::bind("0.0.0.0:15000").unwrap();

        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => {
                    self.handle_connection(stream);
                },
                Err(_) => {
                    println!("--> Incoming connection was dropped.");
                }
            }
        }
    }



    fn handle_connection(&mut self, stream : TcpStream) {

        let mut connection = Connection::new(stream);


        let request_str = connection.read_data();
        
        if !request_str.is_ok() {
            println!("--> Connection reset by peer.");
            return;
        }

        let request = HttpParser::parse(&request_str.unwrap());

        match request {
            Err(msg) => {
                println!("  ! Error: {}", msg);
            },
            Ok(HttpRequest::GET(data)) => self.handle_get_messages(connection, data),
            Ok(HttpRequest::POST(data)) => self.handle_post_message(&mut connection, data),
        }
        println!("  > Connection closed.");
    }

    fn handle_get_messages(&mut self, connection : Connection, data : HttpRequestContent) {
        //println!("    GET {}\n    Body: {}", data.location, data.body);
        self.waiting_connections.push(connection);
    }

    fn handle_message_available(&mut self, message : String){

        let response = HttpFormatter::ok_with_body(&message);

        let mut conn_res = self.waiting_connections.pop();
        while conn_res.is_some() {
            let mut conn = conn_res.unwrap();
            conn.write_data(&response);
            conn_res = self.waiting_connections.pop();
        }
    }

    fn handle_post_message(&mut self, connection : &mut Connection, data : HttpRequestContent) {
        //println!("    POST {}\n    Body: {}", data.location, data.body);
        println!("  > Received message: {}", data.body);

        let response = HttpFormatter::ok();
        connection.write_data(&response);

        self.handle_message_available(data.body);
    }
}

struct Connection {
    stream : TcpStream,
}

impl Connection {
    fn new(stream : TcpStream) -> Connection {
        Connection{
            stream
        }
    }

    fn read_data(&mut self) -> Result<String, io::Error> {
        let mut buffer = [0; 512];

        match self.stream.read(&mut buffer) {
            Ok(_) => {
                let data = String::from_utf8_lossy(&buffer[..]);
                Ok(data.to_string())
            },
            Err(e) => Err(e)
        }
    }

    fn write_data(&mut self, data : &str) {
        self.stream.write(data.as_bytes()).unwrap();
        self.stream.flush().unwrap();
    }
}