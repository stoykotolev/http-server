use std::{
    env::args,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub user_agent: String,
    pub body: String,
}

impl TryFrom<BufReader<&TcpStream>> for Request {
    type Error = &'static str;
    fn try_from(mut buffer: BufReader<&TcpStream>) -> Result<Self, Self::Error> {
        let request =
            String::from_utf8(buffer.fill_buf().expect("couldn't read buffer").to_vec()).unwrap();

        let content_parts: Vec<&str> = request.split("\r\n").collect();
        println!("{:?}", content_parts);
        let status_line: Vec<&str> = content_parts[0].split_whitespace().collect();
        let user_agent_vec = content_parts[2].split_whitespace().collect::<Vec<&str>>();
        let user_agent = if user_agent_vec.len() < 2 {
            "".to_string()
        } else {
            user_agent_vec[1].to_string()
        };
        let request_body = if content_parts.len() < 7 {
            "".to_string()
        } else {
            content_parts[6].to_string()
        };

        Ok(Self {
            method: status_line[0].to_string(),
            path: status_line[1].to_string(),
            version: status_line[2].to_string(),
            user_agent,
            body: request_body,
        })
    }
}

#[derive(Debug)]
pub enum ResponseHeader {
    HttpOk,
    HttpNotFound,
    HttpBad,
    HttpMethodNotAllowed,
    HttpCreated,
}

impl ResponseHeader {
    pub fn get(response: ResponseHeader) -> String {
        match response {
            ResponseHeader::HttpOk => "200 OK".to_string(),
            ResponseHeader::HttpNotFound => "404 Not Found".to_string(),
            ResponseHeader::HttpBad => "500 Bad".to_string(),
            ResponseHeader::HttpMethodNotAllowed => "405 Method Not Allowed".to_string(),
            ResponseHeader::HttpCreated => "201 Created".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub version: String,
    pub header: String,
    pub content_type: String,
    pub content_length: usize,
    pub body: String,
}

impl Response {
    pub fn new(request: Request) -> Self {
        let (_, rest) = request.path.split_at(1);
        let (sub_path, remaining_paths) = rest.split_once("/").unwrap_or((rest, ""));
        match request.method.as_ref() {
            "GET" => {
                if rest.is_empty() {
                    return Self {
                        version: request.version,
                        header: ResponseHeader::get(ResponseHeader::HttpOk),
                        body: String::new(),
                        content_type: String::new(),
                        content_length: 0,
                    };
                }
                match sub_path {
                    "echo" => Self {
                        version: request.version,
                        header: ResponseHeader::get(ResponseHeader::HttpOk),
                        content_type: "text/plain".to_string(),
                        content_length: remaining_paths.len(),
                        body: remaining_paths.to_string(),
                    },
                    "user-agent" => Self {
                        version: request.version,
                        header: ResponseHeader::get(ResponseHeader::HttpOk),
                        content_type: "text/plain".to_string(),
                        content_length: request.user_agent.len(),
                        body: request.user_agent,
                    },
                    "files" => {
                        let directory = args()
                            .skip_while(|arg| arg != "--directory")
                            .nth(1)
                            .unwrap_or_else(|| ".".to_string());
                        let file = fs::read_to_string(format!("{}/{}", directory, remaining_paths));
                        match file {
                            Ok(opened_file) => Self {
                                version: request.version,
                                header: ResponseHeader::get(ResponseHeader::HttpOk),
                                content_type: "application/octet-stream".to_string(),
                                content_length: opened_file.len(),
                                body: opened_file,
                            },
                            Err(_) => Self {
                                version: request.version,
                                header: ResponseHeader::get(ResponseHeader::HttpNotFound),
                                body: String::new(),
                                content_type: String::new(),
                                content_length: 0,
                            },
                        }
                    }
                    _ => Self {
                        version: request.version,
                        header: ResponseHeader::get(ResponseHeader::HttpNotFound),
                        body: String::new(),
                        content_type: String::new(),
                        content_length: 0,
                    },
                }
            }
            "POST" => match sub_path {
                "files" => {
                    let directory = args()
                        .skip_while(|arg| arg != "--directory")
                        .nth(1)
                        .unwrap_or_else(|| ".".to_string());
                    let mut file = File::create(format!("{}/{}", directory, remaining_paths))
                        .expect("couldn't create file");
                    file.write_all(request.body.as_bytes())
                        .expect("couldn't write to file");
                    Self {
                        version: request.version,
                        header: ResponseHeader::get(ResponseHeader::HttpCreated),
                        body: String::new(),
                        content_type: String::new(),
                        content_length: 0,
                    }
                }
                _ => Self {
                    version: request.version,
                    header: ResponseHeader::get(ResponseHeader::HttpNotFound),
                    body: String::new(),
                    content_type: String::new(),
                    content_length: 0,
                },
            },
            _ => Self {
                version: request.version,
                header: ResponseHeader::get(ResponseHeader::HttpMethodNotAllowed),
                body: String::new(),
                content_type: String::new(),
                content_length: 0,
            },
        }
    }

    pub fn format_response(&self) -> String {
        format!(
            "{} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            self.version, self.header, self.content_type, self.content_length, self.body
        )
    }
}
