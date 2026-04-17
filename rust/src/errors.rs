use thiserror::Error;

#[derive(Debug, Error)]
pub enum DynDnsError {
    #[error("No token provided for authenticating with the API.")]
    MissingToken,

    #[error("No endpoint provided")]
    #[allow(dead_code)]
    MissingEndpoint,

    #[error("{0:?} is not a valid IP address")]
    InvalidIp(String),

    #[error("Can't update dyndns entry, no IPs provided")]
    NoIp,

    #[error("Can't update dyndns entry, no valid domain provided")]
    NoDomain,

    #[error("HTTP Request returned with Status Code {status}, expected 200. Response body: {body}")]
    HttpRequestFailed { status: u16, body: String },

    #[error("Remote api call returned without success, response: {0}")]
    OperationFailed(String),

    #[error("Unknown log level {0}")]
    UnknownLogLevel(String),

    #[error("Interval is too short, needs to be at least 30s, current {0}")]
    InvalidInterval(String),

    #[error("SSL is enabled but certificate and/or private key are missing")]
    #[allow(dead_code)]
    IncompleteSSLConfig,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yml::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
