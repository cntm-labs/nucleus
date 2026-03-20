use super::{HttpClient, PaginatedResponse};
use crate::verify::NucleusError;
use serde::Deserialize;
use std::sync::Arc;

/// A Nucleus organisation returned by the Admin API.
#[derive(Debug, Clone, Deserialize)]
pub struct NucleusOrg {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Optional parameters for listing organisations.
#[derive(Debug, Default)]
pub struct ListOrgsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

/// Admin Orgs API.
#[derive(Clone)]
pub struct OrgsApi {
    http: Arc<HttpClient>,
}

impl OrgsApi {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// Get a single organisation by ID.
    pub async fn get(&self, org_id: &str) -> Result<NucleusOrg, NucleusError> {
        self.http.get(&format!("/orgs/{org_id}")).await
    }

    /// List organisations with optional pagination.
    pub async fn list(
        &self,
        params: Option<ListOrgsParams>,
    ) -> Result<PaginatedResponse<NucleusOrg>, NucleusError> {
        let mut query_parts: Vec<String> = Vec::new();
        if let Some(p) = &params {
            if let Some(limit) = p.limit {
                query_parts.push(format!("limit={limit}"));
            }
            if let Some(cursor) = &p.cursor {
                query_parts.push(format!("cursor={cursor}"));
            }
        }

        let path = if query_parts.is_empty() {
            "/orgs".to_string()
        } else {
            format!("/orgs?{}", query_parts.join("&"))
        };

        self.http.get(&path).await
    }
}
