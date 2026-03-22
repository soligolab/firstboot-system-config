use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::backend::NativeHostService;
use crate::models::{ApplyConfigurationRequest, SaveTimeSettingsRequest, TimeState};

#[derive(Clone, Debug)]
pub struct ApiClient {
    host: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let host = base_url
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_string();
        Self { host }
    }

    pub fn get_time(&self) -> Result<TimeState, String> {
        let body = self.send_request("GET", "/api/time", "")?;
        TimeState::from_body(&body)
    }

    pub fn save_time_settings(&self, request: &SaveTimeSettingsRequest) -> String {
        self.send_request("POST", "/api/time", &request.to_body())
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    pub fn apply_configuration(&self, request: &ApplyConfigurationRequest) -> String {
        self.send_request("POST", "/api/configuration", &request.to_body())
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    pub fn backup_recovery(&self) -> String {
        self.send_request("POST", "/api/backup-recovery", "")
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    pub fn factory_reset(&self) -> String {
        self.send_request("POST", "/api/factory-reset", "")
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    fn send_request(&self, method: &str, path: &str, body: &str) -> Result<String, String> {
        let mut stream = TcpStream::connect(&self.host).map_err(|err| err.to_string())?;
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.host,
            body.len(),
            body
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|err| err.to_string())?;
        stream.flush().map_err(|err| err.to_string())?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|err| err.to_string())?;
        parse_http_response(&response)
    }
}

pub fn spawn_server(addr: String, service: NativeHostService) {
    thread::spawn(move || run_server(addr, service));
}

pub fn run_server(addr: String, service: NativeHostService) {
    let listener =
        TcpListener::bind(&addr).unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));
    let service = Arc::new(service);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let service = Arc::clone(&service);
                thread::spawn(move || {
                    if let Err(err) = handle_connection(stream, service) {
                        eprintln!("api connection error: {err}");
                    }
                });
            }
            Err(err) => eprintln!("api accept error: {err}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream, service: Arc<NativeHostService>) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|err| err.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|err| err.to_string())?;

    if request_line.trim().is_empty() {
        return Ok(());
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or_else(|| "missing method".to_string())?;
    let path = parts.next().ok_or_else(|| "missing path".to_string())?;

    let mut content_length = 0usize;
    loop {
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|err| err.to_string())?;
        if header == "\r\n" || header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body_bytes = vec![0u8; content_length];
    reader
        .read_exact(&mut body_bytes)
        .map_err(|err| err.to_string())?;
    let body = String::from_utf8(body_bytes).map_err(|err| err.to_string())?;

    let (status, response_body) = route_request(method, path, &body, &service);
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|err| err.to_string())?;
    stream.flush().map_err(|err| err.to_string())
}

fn route_request(
    method: &str,
    path: &str,
    body: &str,
    service: &NativeHostService,
) -> (&'static str, String) {
    match (method, path) {
        ("GET", "/api/time") => ("200 OK", service.current_time().to_body()),
        ("POST", "/api/time") => match SaveTimeSettingsRequest::from_body(body) {
            Ok(payload) => ("200 OK", service.save_time_settings(payload)),
            Err(err) => ("400 Bad Request", format!("invalid request: {err}")),
        },
        ("POST", "/api/configuration") => match ApplyConfigurationRequest::from_body(body) {
            Ok(payload) => ("200 OK", service.apply_configuration(payload)),
            Err(err) => ("400 Bad Request", format!("invalid request: {err}")),
        },
        ("POST", "/api/backup-recovery") => ("200 OK", service.backup_recovery()),
        ("POST", "/api/factory-reset") => ("200 OK", service.factory_reset()),
        _ => ("404 Not Found", "not found".to_string()),
    }
}

fn parse_http_response(response: &str) -> Result<String, String> {
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| "invalid HTTP response".to_string())?;
    let status_line = head
        .lines()
        .next()
        .ok_or_else(|| "missing status line".to_string())?;
    if !status_line.contains(" 200 ") {
        return Err(format!("server returned {status_line}: {body}"));
    }
    Ok(body.to_string())
}
