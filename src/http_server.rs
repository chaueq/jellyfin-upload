use std::{collections::HashMap, io::{BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, sync::mpsc::channel, thread::{self, sleep}, time::Duration};

use crate::{config::{Config, ProgramFile}, http_handler, keystore::{self, Keystore, AUTH_HEADER}, module::{Module, ModuleMgmtSignal}};

const MAX_UPLOAD_SIZE: usize = 104857600; //10 MB
const STACK_SIZE: usize = MAX_UPLOAD_SIZE * 3;

pub fn start(config: Config) -> Module {
    let (mgmt_sender, mgmt_receiver) = channel::<ModuleMgmtSignal>();

    let handle = thread::Builder::new()
    .name("http server".to_string())
    .stack_size(STACK_SIZE)
    .spawn(move || {
        let listener: TcpListener = TcpListener::bind(config.get_net_address()).unwrap();
        listener.set_nonblocking(true).unwrap();

        let mut default_headers:HashMap<String, String> = HashMap::new();
        default_headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
        default_headers.insert("Server".to_string(), "antispam".to_string());
        default_headers.insert("X-Frame-Options".to_string(), "SAMEORIGIN".to_string());
        default_headers.insert("X-Xss-Protection".to_string(), "1; mode=block".to_string());

        let mut keystore = Keystore::from_file(&config.get_path(ProgramFile::Keystore));

        loop {
            if let Ok(req) = mgmt_receiver.try_recv() {
                match req {
                    ModuleMgmtSignal::Stop => {
                        break;
                    }
                    ModuleMgmtSignal::Refresh => {
                        keystore = Keystore::from_file(&config.get_path(ProgramFile::Keystore));
                    }
                }
            }
            else if let Ok((mut stream, _)) = listener.accept() {
                let config = config.clone();
                let default_headers = default_headers.clone();
                let keystore = keystore.clone();
                let _ = thread::Builder::new()
                .stack_size(STACK_SIZE)
                .spawn(move || {

                    let mut resp: HttpResponse = match parse_http_request(&mut stream) {
                            Some(request) => {
                                match request.method {
                                    HttpMethod::GET => {
                                        http_handler::serve_file(request, &config)
                                    }
                                    HttpMethod::POST | HttpMethod::PUT => {
                                        match request.headers.get(AUTH_HEADER) {
                                            Some(key) => {
                                                match keystore.authorize(key, keystore::Permission::Upload) {
                                                    true => {
                                                        http_handler::upload_file(request, &config)
                                                    }
                                                    false => {
                                                        HttpResponse::minimal(403)
                                                    }
                                                }

                                            }
                                            None => {
                                                HttpResponse::minimal(401)
                                            }
                                        }
                                    }
                                    HttpMethod::OPTIONS => {
                                        let mut headers: HashMap<String, String> = HashMap::new();
                                        headers.insert("Access-Control-Allow-Methods".to_string(), "POST, GET, OPTIONS".to_string());
                                        headers.insert("Access-Control-Allow-Headers".to_string(), "X-Apikey, Content-Type".to_string());
                                        headers.insert("Access-Control-Max-Age".to_string(), "86400".to_string());
                            
                                        HttpResponse {
                                            status: 204,
                                            headers,
                                            body: None
                                        }
                                    },
                                    _ => {
                                        HttpResponse::minimal(405)
                                    }
                                }                            
                            },
                            None => {
                                HttpResponse::minimal(400)
                            }
                        };

                        for (k, v) in &default_headers {
                            resp.headers.insert(k.clone(), v.clone());
                        }

                        
                        send_http_response(resp, &mut stream);
                }).unwrap();
            }
            else {
                sleep(Duration::from_millis(10));
            }
        }
    });

    Module::new(handle.unwrap(), mgmt_sender)
}

fn parse_http_request(mut stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut reader: BufReader<&mut TcpStream> = BufReader::with_capacity(MAX_UPLOAD_SIZE,&mut stream);
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut first_line = String::new();
    
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(_) => {},
            Err(_) => {return None;}
        };

        line = match line.strip_suffix("\r\n") {
            Some(text) => {text.to_string()},
            None => {line}
        };
        if line.len() == 0 {
            break;
        }
        
        if first_line.len() == 0 {
            first_line = line;
        }
        else {
            let header: Vec<_> = line.split(':').collect();
            
            headers.insert(
                header[0].to_ascii_lowercase().to_string(),
                header[1].trim_start().to_string()
            );
        }
    }

    let body = match &headers.get("content-length") {
        Some(len) => {
            let len = match len.parse::<usize>() {
                Ok(len) => {
                    if len > MAX_UPLOAD_SIZE {
                        return None;
                    }
                    len
                }
                Err(_) => {return None;}
            };
            let mut body_buf = vec![0; len];
            match reader.read_exact(&mut body_buf) {
                Ok(_) => {
                    Some(body_buf)
                }
                Err(_) => {return None;}
            }
        },
        None => {None}
    };

    let request_components: Vec<_> = first_line.split_ascii_whitespace().collect();
    
    if request_components.len() < 2 {
        return None;
    }
    
    println!("{}\t{}", request_components[0], request_components[1]);

    Some(HttpRequest {
        method: match request_components[0] {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "HEAD" => HttpMethod::HEAD,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "CONNECT" => HttpMethod::CONNECT,
            "OPTIONS" => HttpMethod::OPTIONS,
            "TRACE" => HttpMethod::TRACE,
            "PATCH" => HttpMethod::PATCH,
            _ => HttpMethod::Invalid
        },
        path: request_components[1].to_string(),
        headers,
        body
    })
}

fn send_http_response(response: HttpResponse, stream: &mut TcpStream) {
    let mut r = String::new();
    let status_line = "HTTP/1.1 ".to_owned() + match &response.status {
        100 => "100 Continue",
        101 => "101 Switching Protocols",
        102 => "102 Processing",
        103 => "103 Early Hints",
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        203 => "203 Non-Authoritative Information",
        204 => "204 No Content",
        205 => "205 Reset Content",
        206 => "206 Partial Content",
        207 => "207 Multi-Status",
        208 => "208 Already Reported",
        218 => "218 This is fine",
        226 => "226 IM Used",
        300 => "300 Multiple Choices",
        301 => "301 Moved Permanently",
        302 => "302 Found",
        303 => "303 See Other",
        304 => "304 Not Modified",
        306 => "306 Switch Proxy",
        307 => "307 Temporary Redirect",
        308 => "308 Resume Incomplete",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        402 => "402 Payment Required",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        405 => "405 Method Not Allowed",
        406 => "406 Not Acceptable",
        407 => "407 Proxy Authentication Required",
        408 => "408 Request Timeout",
        409 => "409 Conflict",
        410 => "410 Gone",
        411 => "411 Length Required",
        412 => "412 Precondition Failed",
        413 => "413 Request Entity Too Large",
        414 => "414 Request-URI Too Long",
        415 => "415 Unsupported Media Type",
        416 => "416 Requested Range Not Satisfiable",
        417 => "417 Expectation Failed",
        418 => "418 I'm a teapot",
        419 => "419 Page Expired",
        420 => "420 Method Failure",
        421 => "421 Misdirected Request",
        422 => "422 Unprocessable Entity",
        423 => "423 Locked",
        424 => "424 Failed Dependency",
        426 => "426 Upgrade Required",
        428 => "428 Precondition Required",
        429 => "429 Too Many Requests",
        431 => "431 Request Header Fields Too Large",
        440 => "440 Login Time-out",
        444 => "444 Connection Closed Without Response",
        449 => "449 Retry With",
        450 => "450 Blocked by Windows Parental Controls",
        451 => "451 Unavailable For Legal Reasons",
        494 => "494 Request Header Too Large",
        495 => "495 SSL Certificate Error",
        496 => "496 SSL Certificate Required",
        497 => "497 HTTP Request Sent to HTTPS Port",
        498 => "498 Invalid Token",
        499 => "499 Client Closed Request",
        501 => "501 Not Implemented",
        502 => "502 Bad Gateway",
        503 => "503 Service Unavailable",
        504 => "504 Gateway Timeout",
        505 => "505 HTTP Version Not Supported",
        506 => "506 Variant Also Negotiates",
        507 => "507 Insufficient Storage",
        508 => "508 Loop Detected",
        509 => "509 Bandwidth Limit Exceeded",
        510 => "510 Not Extended",
        511 => "511 Network Authentication Required",
        520 => "520 Unknown Error",
        521 => "521 Web Server Is Down",
        522 => "522 Connection Timed Out",
        523 => "523 Origin Is Unreachable",
        524 => "524 A Timeout Occurred",
        525 => "525 SSL Handshake Failed",
        526 => "526 Invalid SSL Certificate",
        527 => "527 Railgun Listener to Origin Error",
        530 => "530 Origin DNS Error",
        598 => "598 Network Read Timeout Error",
        _ => "500 Internal Server Error"
    };

    let body = match &response.body {
        Some(b) => {b.to_string()},
        None => {response.status.to_string().clone()}
    };

    r.push_str(&status_line);
    r.push_str("\r\n");

    for (k, v) in &response.headers {
        if k == "Content-Length" {
            continue;
        }
        r.push_str(k);
        r.push_str(": ");
        r.push_str(v);
        r.push_str("\r\n");
    }

    r.push_str(&format!("Content-Length: {}\r\n", &body.len()));
    r.push_str("\r\n");
    r.push_str(&body);

    stream.write_all(r.as_bytes()).unwrap();
}

#[derive(PartialEq)]
pub enum HttpMethod {
    Invalid,
    GET,
    POST,
    HEAD,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>
}

pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpResponse {
    pub fn minimal(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::<String,String>::new(),
            body: None
        }
    }

    pub fn normal(body: String) -> Self {
        let mut x = Self::minimal(200);
        x.body = Some(body);
        x
    }
}