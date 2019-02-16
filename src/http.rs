use regex::Captures;
use regex::RegexBuilder;

pub struct HttpFormatter { }

impl HttpFormatter {
    pub fn ok_with_body(body : &str) -> String {
        let mut headers = HttpFormatter::format_headers("200 OK");
        headers.push_str(body);
        println!("  > Writing response: {} chars.", headers.len());
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
    pub fn parse(request_str : &str) -> Result<HttpRequest, &'static str> {

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

pub enum HttpRequest {
    GET(HttpRequestContent),
    POST(HttpRequestContent),
}

pub struct HttpRequestContent{
    pub location : String,
    pub headers: String,
    pub body : String,
}
