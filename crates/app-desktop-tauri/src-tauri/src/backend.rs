use crate::models::{CrearSalaRequest, SalaDto};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct BackendApi {
    client: Client,
    base_url: String,
    token: Arc<Mutex<Option<String>>>,
}

impl BackendApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let normalized = base_url.trim_end_matches('/').to_string();

        Self {
            client: Client::new(),
            base_url: normalized,
            token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn set_token(&self, token: Option<String>) {
        *self.token.lock().unwrap() = token;
    }

    fn build_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Ok(token_guard) = self.token.lock() {
            if let Some(ref token) = *token_guard {
                return request.header("Authorization", format!("Bearer {}", token));
            }
        }
        request
    }

    pub async fn listar_salas(&self) -> Result<Vec<SalaDto>, String> {
        let url = self.endpoint("/salas");
        log_request("GET", &url);
        let mut req = self.client.get(&url);
        req = self.build_request(req);
        let response = req.send().await.map_err(to_string)?;
        log_status("GET", &url, response.status());
        parse_response(response).await
    }

    pub async fn crear_sala(&self, request: CrearSalaRequest) -> Result<SalaDto, String> {
        let url = self.endpoint("/salas");
        log_request("POST", &url);
        let mut req = self.client.post(&url).json(&request);
        req = self.build_request(req);
        let response = req.send().await.map_err(to_string)?;
        log_status("POST", &url, response.status());
        parse_response(response).await
    }

    pub async fn obtener_sala(&self, id: &str) -> Result<Option<SalaDto>, String> {
        let url = self.endpoint(&format!("/salas/{id}"));
        log_request("GET", &url);
        let response = self.client.get(&url).send().await.map_err(to_string)?;
        log_status("GET", &url, response.status());

        match response.status() {
            StatusCode::NOT_FOUND => Ok(None),
            _ => parse_response(response).await.map(Some),
        }
    }

    pub async fn activar_sala(&self, id: &str) -> Result<SalaDto, String> {
        let url = self.endpoint(&format!("/salas/{id}/activar"));
        log_request("PUT", &url);
        let mut req = self.client.put(&url);
        req = self.build_request(req);
        let response = req.send().await.map_err(to_string)?;
        log_status("PUT", &url, response.status());
        parse_response(response).await
    }

    pub async fn desactivar_sala(&self, id: &str) -> Result<SalaDto, String> {
        let url = self.endpoint(&format!("/salas/{id}/desactivar"));
        log_request("PUT", &url);
        let mut req = self.client.put(&url);
        req = self.build_request(req);
        let response = req.send().await.map_err(to_string)?;
        log_status("PUT", &url, response.status());
        parse_response(response).await
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }
}

fn to_string<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

fn log_request(method: &str, url: &str) {
    println!("[app-desktop-tauri][req ] {method} {url}");
}

fn log_status(method: &str, url: &str, status: StatusCode) {
    println!(
        "[app-desktop-tauri][resp] {method} {url} -> {} {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or("Estado desconocido")
    );
}

async fn parse_response<T>(response: reqwest::Response) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status.is_success() {
        response
            .json::<T>()
            .await
            .map_err(|err| format!("Error parseando respuesta: {err}"))
    } else {
        let message = parse_error_body(response).await;
        Err(format!("{} {}", status.as_u16(), message))
    }
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    error: Option<String>,
    errors: Option<Vec<String>>,
}

async fn parse_error_body(response: reqwest::Response) -> String {
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Error del backend")
        .to_string();

    match response.text().await {
        Ok(body) => {
            if body.is_empty() {
                return status_text;
            }

            if let Ok(parsed) = serde_json::from_str::<ErrorBody>(&body) {
                if let Some(errors) = parsed.errors {
                    if !errors.is_empty() {
                        return errors.join(", ");
                    }
                }

                if let Some(error) = parsed.error {
                    return error;
                }
            }

            body
        }
        Err(err) => format!("{status_text}: {err}"),
    }
}
