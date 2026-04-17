use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    pub errors: Vec<CloudflareMessage>,
    #[allow(dead_code)]
    pub messages: Vec<CloudflareMessage>,
    pub success: bool,
    pub result: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareMessage {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareZone {
    pub id: String,
}

/// Used for both GET responses and POST/PUT request bodies.
/// Fields with `skip_serializing_if` replace Go's `omitempty`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CloudflareRecord {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub content: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub name: String,
    #[serde(skip_serializing_if = "is_false", default)]
    pub proxied: bool,
    #[serde(rename = "type", skip_serializing_if = "String::is_empty", default)]
    pub record_type: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub comment: String,
    #[serde(skip_serializing_if = "is_zero", default)]
    pub ttl: u32,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub id: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub modified_on: String,
}

fn is_false(v: &bool) -> bool {
    !v
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}
