use thiserror::Error;

#[derive(uniffi::Error, Error, Debug, PartialEq)]
pub enum MobileError {
    #[error("Rule file not found: {0}")]
    FileNotFound(String),
    #[error("Invalid rule manifest: {0}")]
    InvalidManifest(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Internal engine error: {0}")]
    InternalError(String),
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum MobileVerdict {
    Direct,
    DpiEvasiveDirect,
    Zapret2 { preset: String },
    StreamProxy { mode: String },
    Proxy { gateway_id: String },
    DnsOverride { ips: Vec<String> },
    Block { reason: Option<String> },
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct MobileRuleInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub match_count: u32,
}

#[derive(uniffi::Record, Debug, Clone, Default, PartialEq)]
pub struct MobileMetrics {
    pub total_queries: u64,
    pub blocked_queries: u64,
    pub proxied_queries: u64,
    pub dpi_evasive_queries: u64,
    pub direct_queries: u64,
    pub memory_bytes: u64,
}
