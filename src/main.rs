mod http;

use std::io;
use std::io::prelude::*;

use std::net::TcpListener;
use std::net::TcpStream;
use std::net::Shutdown;
use std::net::SocketAddr;

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
        let request = HttpParser::parse(&mut connection);
        match request {
            Err(msg) => {
                println!("  ! Error: {}", msg);
                match connection.shutdown() {
                    Ok(()) => {},
                    Err(_) => println!("  ! Additionally, could not shutdown socket."),
                };
            },
            Ok(HttpRequest::GET(data)) => self.handle_get_messages(connection, data),
            Ok(HttpRequest::POST(data)) => {
                self.handle_post_message(&mut connection, data);
            }
        }
    }

    fn handle_get_messages(&mut self, connection : Connection, _data : HttpRequestContent) {
        //println!("    GET {}\n    Body: {}", data.location, data.body);
        self.waiting_connections.push(connection);
    }

    fn handle_message_available(&mut self, message : String){

        let response = HttpFormatter::ok_with_body(&message);

        let mut conn_res = self.waiting_connections.pop();
        while conn_res.is_some() {
            let mut conn = conn_res.unwrap();
            println!("  > Forwarding message to {}", conn.peer_addr());
            conn.write_data(&response);
            conn_res = self.waiting_connections.pop();
        }
    }

    fn handle_post_message(&mut self, connection : &mut Connection, data : HttpRequestContent) {
        //println!("    POST {}\n    Body: {}", data.location, data.body);
        println!("--> Received message: \"{}\", {} bytes", data.body, data.body.len());

        let response = HttpFormatter::ok();
        connection.write_data(&response);

        self.handle_message_available(data.body);
    }
}

pub struct Connection {
    stream : TcpStream,
}

const BUF_SIZE : usize = 512;

impl Connection {
    fn new(stream : TcpStream) -> Connection {
        Connection{
            stream
        }
    }

    fn read_more(&mut self, num_bytes : usize) -> Result<String, io::Error> {
        println!("    Reading an additional {} bytes...", num_bytes);
        match self.read_string() {
            Ok(data) => {
                if data.len() >= num_bytes {
                    println!("    Done. Read {} more bytes (needed {}).", data.len(), num_bytes);
                    Ok(data)
                } else {
                    println!("    Not yet done. Read {} more bytes (needed {}).", data.len(), num_bytes);
                    self.read_more(num_bytes - data.len())
                }
            },
            Err(err) => Err(err),
        }
    }

    fn read_string(&mut self) -> Result<String, io::Error> {

        let mut buffer = [0; BUF_SIZE];

        match self.stream.read(&mut buffer) {
            Ok(read) => {
                let mut data = String::from_utf8_lossy(&buffer[..read]).to_string();
                println!("DATA: \"{}\"", data);
                if read == BUF_SIZE {
                    println!("  > Buffer is full, there is more data.");
                    match self.read_string() {
                        Ok(next_data) => {
                            data.push_str(&next_data);
                            Ok(data)
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(data)
                }
            },
            Err(e) => Err(e)
        }
    }

    fn write_data(&mut self, data : &str) {
        match self.stream.write(data.as_bytes()) {
            Ok(_) => {},
            Err(_) => {
                println!("--> Write failed: connection closed.");
                return;
            },
        }
        match self.stream.flush() {
            Ok(_) => {},
            Err(_) => println!("--> Flush failed: connection closed."), 
        }
    }

    fn peer_addr(&self) -> SocketAddr {
         self.stream.peer_addr().unwrap()
    }

    fn shutdown(&mut self) -> Result<(), io::Error>{
        self.stream.shutdown(Shutdown::Both)
    }
}