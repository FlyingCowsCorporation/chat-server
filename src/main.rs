use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::result::Result;
use regex::Regex;
use regex::Captures;
use regex::RegexBuilder;

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
            println!("GET {}: \n{}\n{}", data.location, data.headers, data.body);
            let response = HttpFormatter::ok("GET ACCEPTED");
            connection.write_data(&response);
        },
        Ok(HttpRequest::POST(data)) => {
            println!("  {}: \n{}\n{}", data.location, data.headers, data.body);
            let response = HttpFormatter::ok("POST ACCEPTED");
            connection.write_data(&response);
        },
    }
    println!("  > Connection closed.");
}

enum HttpRequest {
    GET(HttpRequestContent),
    POST(HttpRequestContent),
}

struct HttpRequestContent{
    location : String,
    headers: String,
    body : String,
}

struct HttpParser {}

impl HttpParser {
    fn parse(request_str : &str) -> Result<HttpRequest, &'static str> {

        let re = RegexBuilder::new(r"([^\s]+) ([^\s]+) HTTP/.\..\r\n(.*)\r\n\r\n(.*)")
            .dot_matches_new_line(true)
            .build().unwrap();
        
        match re.captures(request_str) {
            Some(captures) => HttpParser::match_to_request(captures),
            None => {
                Err("Invalid request.")
            }
        }
    }

    fn match_to_request(captures : Captures) -> Result<HttpRequest, &'static str> {
        let method = captures.get(1).map_or("", |m| m.as_str());
        let location = captures.get(2).map_or("", |m| m.as_str()).to_owned();
        let headers = captures.get(3).map_or("", |m| m.as_str()).to_owned();
        let body = captures.get(4).map_or("", |m| m.as_str()).to_owned();

        Ok(match method {
            "GET" => HttpRequest::GET(HttpRequestContent {location, headers, body}),
            "POST" => HttpRequest::POST(HttpRequestContent {location, headers, body}),
            _ => {
                return Err("Invalid method");
            }
        })
    }
}

struct HttpFormatter { }

impl HttpFormatter {
    fn ok(body : &str) -> String {
        let mut headers = HttpFormatter::format_headers("200 OK");
        headers.push_str(body);
        headers
    }

    fn format_headers(status : &str) -> String{
        let mut headers = String::from("HTTP/1.1 ");
        headers.push_str(status);
        headers.push_str("\r\n\r\n");
        headers
    }
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