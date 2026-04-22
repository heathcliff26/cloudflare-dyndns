use serde::{Deserialize, Serialize};

pub const DEFAULT_API_URL: &str = "https://api.cloudflare.com/client/v4";

/// Response wrapper for Cloudflare API responses.
#[derive(Clone, Serialize, Deserialize)]
pub struct CloudflareResponse<T> {
    pub success: bool,
    pub errors: Vec<CloudflareMessage>,
    pub messages: Vec<CloudflareMessage>,
    pub result: Option<T>,
}

/// Message structure for Cloudflare API responses.
#[derive(Clone, Serialize, Deserialize)]
pub struct CloudflareMessage {
    pub code: u64,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Inactive,
    Disabled,
}

/// Result for Verify Token endpoint.
#[derive(Clone, Serialize, Deserialize)]
pub struct VerifyTokenResult {
    pub id: String,
    pub status: Status,
}

/// Cloudflare dns zone
#[derive(Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub status: Status,
}

/// Cloudflare dns record
#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    pub name: String,
    pub ttl: u64,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub proxied: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
}

/// Record type, either A or AAAA
pub enum RecordType {
    A,
    #[allow(clippy::upper_case_acronyms)]
    AAAA,
}
