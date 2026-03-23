use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::backend::NativeHostService;
use crate::models::{ApplyConfigurationRequest, SaveTimeSettingsRequest, TimeState};
use crate::web;

/// Client HTTP minimale senza dipendenze esterne.
///
/// È sufficiente per il dialogo locale GUI <-> backend e mantiene la demo facile da
/// compilare anche in ambienti embedded o molto controllati.
#[derive(Clone, Debug)]
pub struct ApiClient {
    host: String,
}

impl ApiClient {
    /// Accetta un URL base come `http://127.0.0.1:7878` e ne estrae la parte host:port
    /// da usare nelle connessioni TCP raw.
    pub fn new(base_url: String) -> Self {
        let host = base_url
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_string();
        Self { host }
    }

    /// Recupera lo stato orario corrente dal backend.
    pub fn get_time(&self) -> Result<TimeState, String> {
        let body = self.send_request("GET", "/api/time", "")?;
        TimeState::from_body(&body)
    }

    /// Invia al backend le nuove impostazioni di data/ora/timezone.
    pub fn save_time_settings(&self, request: &SaveTimeSettingsRequest) -> String {
        self.send_request("POST", "/api/time", &request.to_body())
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    /// Invia la configurazione completa dei profili utente.
    pub fn apply_configuration(&self, request: &ApplyConfigurationRequest) -> String {
        self.send_request("POST", "/api/configuration", &request.to_body())
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    /// Richiede al backend l'esecuzione del flusso di backup recovery.
    pub fn backup_recovery(&self) -> String {
        self.send_request("POST", "/api/backup-recovery", "")
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    /// Richiede al backend l'esecuzione del flusso di factory reset.
    pub fn factory_reset(&self) -> String {
        self.send_request("POST", "/api/factory-reset", "")
            .unwrap_or_else(|err| format!("ERROR: {err}"))
    }

    /// Costruisce una richiesta HTTP/1.1 minimale, apre una connessione TCP al server
    /// locale e restituisce solo il body della risposta se lo status è `200`.
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

/// Avvia il server API in background lasciando libero il thread chiamante.
pub fn spawn_server(addr: String, service: NativeHostService) {
    thread::spawn(move || run_server(addr, service));
}

/// Avvia il listener TCP e delega ogni connessione in ingresso a un thread dedicato.
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

/// Legge una richiesta HTTP raw dalla socket, la instrada e scrive la risposta.
fn handle_connection(mut stream: TcpStream, service: Arc<NativeHostService>) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|err| err.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|err| err.to_string())?;

    // Una connessione chiusa senza dati non è considerata errore applicativo.
    if request_line.trim().is_empty() {
        return Ok(());
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or_else(|| "missing method".to_string())?;
    let path = parts.next().ok_or_else(|| "missing path".to_string())?;

    // Basta conoscere `Content-Length` perché tutti gli endpoint usano body testuali
    // semplici e connessioni a chiusura esplicita.
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

    let response = route_request(method, path, &body, &service);
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        response.content_type,
        response.body.len(),
        response.body
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|err| err.to_string())?;
    stream.flush().map_err(|err| err.to_string())
}

/// Rappresenta la risposta HTTP completa generata dal router interno.
struct HttpResponse {
    status: &'static str,
    content_type: &'static str,
    body: String,
}

/// Router minimale degli endpoint API locali.
fn route_request(
    method: &str,
    path: &str,
    body: &str,
    service: &NativeHostService,
) -> HttpResponse {
    match (method, path) {
        ("GET", "/") => HttpResponse {
            status: "200 OK",
            content_type: "text/html; charset=utf-8",
            body: web::root_page().to_string(),
        },
        ("GET", "/app.css") => HttpResponse {
            status: "200 OK",
            content_type: "text/css; charset=utf-8",
            body: web::app_css().to_string(),
        },
        ("GET", "/app.js") => HttpResponse {
            status: "200 OK",
            content_type: "application/javascript; charset=utf-8",
            body: web::app_js().to_string(),
        },
        ("GET", "/languages.xml") => HttpResponse {
            status: "200 OK",
            content_type: "application/xml; charset=utf-8",
            body: web::languages_xml().to_string(),
        },
        ("GET", "/api/time") => HttpResponse {
            status: "200 OK",
            content_type: "text/plain; charset=utf-8",
            body: service.current_time().to_body(),
        },
        ("POST", "/api/time") => match SaveTimeSettingsRequest::from_body(body) {
            Ok(payload) => HttpResponse {
                status: "200 OK",
                content_type: "text/plain; charset=utf-8",
                body: service.save_time_settings(payload),
            },
            Err(err) => HttpResponse {
                status: "400 Bad Request",
                content_type: "text/plain; charset=utf-8",
                body: format!("invalid request: {err}"),
            },
        },
        ("POST", "/api/configuration") => match ApplyConfigurationRequest::from_body(body) {
            Ok(payload) => HttpResponse {
                status: "200 OK",
                content_type: "text/plain; charset=utf-8",
                body: service.apply_configuration(payload),
            },
            Err(err) => HttpResponse {
                status: "400 Bad Request",
                content_type: "text/plain; charset=utf-8",
                body: format!("invalid request: {err}"),
            },
        },
        ("POST", "/api/backup-recovery") => HttpResponse {
            status: "200 OK",
            content_type: "text/plain; charset=utf-8",
            body: service.backup_recovery(),
        },
        ("POST", "/api/factory-reset") => HttpResponse {
            status: "200 OK",
            content_type: "text/plain; charset=utf-8",
            body: service.factory_reset(),
        },
        _ => HttpResponse {
            status: "404 Not Found",
            content_type: "text/plain; charset=utf-8",
            body: "not found".to_string(),
        },
    }
}

/// Estrae il body da una risposta HTTP e considera valido solo lo status `200 OK`.
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
