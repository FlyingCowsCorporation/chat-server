mod http;
mod connection;

use std::net::TcpListener;
use std::net::TcpStream;

use connection::Connection;

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