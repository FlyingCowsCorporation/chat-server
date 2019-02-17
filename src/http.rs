use std::collections::HashMap;

use regex::Captures;
use regex::RegexBuilder;

use crate::Connection;

pub struct HttpFormatter { }

impl HttpFormatter {
    pub fn ok_with_body(body : &str) -> String {
        let mut headers = HttpFormatter::format_headers("200 OK");
        headers.push_str(body);
        headers
    }

    pub fn ok() -> String {
        HttpFormatter::format_headers("200 OK")
    }

    fn format_headers(status : &str) -> String{
        let mut headers = String::from("HTTP/1.1 ");
        headers.push_str(status);
        headers.push_str("\r\n\r\n");
        headers
    }
}

pub struct HttpParser {}

impl HttpParser {
    pub fn parse(connection : &mut Connection) -> Result<HttpRequest, &'static str> {

        // Read the first data from the connection.
        let request_str = HttpParser::read_until_end(connection);
        if !request_str.is_ok() {
            return Err("Connection reset by peer");
        }
        let mut request_str = request_str.unwrap();
        let request = HttpParser::parse_str(&request_str);

        match request {
            Err(e) => Err(e),
            Ok(HttpRequest::POST(content)) => {
                if content.headers.contains_key("content-length") {
                    let content_length_str = content.headers.get("content-length")
                        .unwrap();
                    
                    match content_length_str.parse::<usize>() {
                        Ok(content_length) => {
                            let actual_length = content.body.len();
                            if actual_length == content_length {
                                Ok(HttpRequest::POST(content))
                            } else {
                                //println!("CRDATA: \"{}\"", content.body);
                                println!("  > We have {} bytes, but we need {}", actual_length, content_length);
                                let additional_content = connection.read_more(content_length - actual_length);
                                match additional_content {
                                    Ok(content) => {
                                        request_str.push_str(&content);
                                        HttpParser::parse_str(&request_str)
                                    },
                                    Err(_) => Err("Expected more content, but read failed.")
                                }
                            }
                        }
                        Err(err) => {
                            println!("String value: \"{:?}\"", content_length_str.as_bytes());
                            println!("Error: {}", err);
                            Err("Could not parse content length")
                        }
                    }
                } else {
                    Ok(HttpRequest::POST(content))
                }
            },
            Ok(HttpRequest::GET(content)) => Ok(HttpRequest::GET(content))
        }
    }

    fn read_until_end(connection : &mut Connection) -> Result<String, &'static str> {
        match connection.read_string() {
            Ok(data) => Ok(data),
            Err(_) => Err("Connection reset by peer."),
        }
    }

    fn parse_str(request_str : &str) -> Result<HttpRequest, &'static str> {

        let re = RegexBuilder::new(r"([^\s]+) ([^\s]+) HTTP/.\..\r\n(.*?)\r\n\r\n(.*)")
            .dot_matches_new_line(true)
            .build().unwrap();

        let captures = re.captures(&request_str);

        if captures.is_none() {
            return Err("Invalid request");
        }
        let captures = captures.unwrap();
        HttpParser::match_to_request(captures)
    }

    fn match_to_request(captures : Captures) -> Result<HttpRequest, &'static str> {
        let method = captures.get(1).map_or("", |m| m.as_str());
        let location = captures.get(2).map_or("", |m| m.as_str()).to_owned();
        let headers_str = captures.get(3).map_or("", |m| m.as_str());
        let body = captures.get(4).map_or("", |m| m.as_str()).to_owned();

        let headers = HttpParser::parse_headers(headers_str);

        Ok(match method {
            "GET" => HttpRequest::GET(HttpRequestContent {location, headers, body}),
            "POST" => HttpRequest::POST(HttpRequestContent {location, headers, body}),
            _ => {
                return Err("Invalid method");
            }
        })
    }

    fn parse_headers(headers_str : &str) -> HashMap<String, String> {
        let re = RegexBuilder::new(r"^([^:]+): ([^\r\n]+)")
            .multi_line(true)
            .build().unwrap();

        let mut headers : HashMap<String, String> = HashMap::new();
        for cap in re.captures_iter(headers_str) {
            headers.insert(
                (&cap[1]).to_lowercase().to_owned(),
                (&cap[2]).to_owned(),
            );
        }
        headers
    }
}

pub enum HttpRequest {
    GET(HttpRequestContent),
    POST(HttpRequestContent),
}

pub struct HttpRequestContent{
    pub location : String,
    pub headers: HashMap<String, String>,
    pub body : String,
}
