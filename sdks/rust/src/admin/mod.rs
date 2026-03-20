mod orgs;
mod users;

pub use orgs::OrgsApi;
pub use users::UsersApi;

use crate::verify::NucleusError;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Shared HTTP client for the Nucleus Admin API.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    secret_key: String,
    http: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: String, secret_key: String) -> Self {
        Self {
            base_url,
            secret_key,
            http: reqwest::Client::new(),
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, NucleusError> {
        self.request(reqwest::Method::GET, path).await
    }

    pub(crate) async fn post<T: DeserializeOwned>(&self, path: &str) -> Result<T, NucleusError> {
        self.request(reqwest::Method::POST, path).await
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<(), NucleusError> {
        let url = format!("{}/api/v1/admin{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .send()
            .await?;

        if resp.status().is_client_error() || resp.status().is_server_error() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(NucleusError::Api {
                status,
                message: body,
            });
        }

        Ok(())
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T, NucleusError> {
        let url = format!("{}/api/v1/admin{}", self.base_url, path);
        let resp = self
            .http
            .request(method, &url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .send()
            .await?;

        if resp.status().is_client_error() || resp.status().is_server_error() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(NucleusError::Api {
                status,
                message: body,
            });
        }

        resp.json().await.map_err(NucleusError::from)
    }
}

/// A paginated response from the Nucleus Admin API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub total_count: Option<u64>,
}
