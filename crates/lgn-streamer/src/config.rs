use serde::Deserialize;
use webrtc::ice_transport::{ice_credential_type::RTCIceCredentialType, ice_server::RTCIceServer};

/// The configuration type for the plugin.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// Whether to enable hardward encoding.
    #[serde(default)]
    pub enable_hw_encoding: bool,

    /// The `WebRTC` configuration.
    #[serde(default)]
    pub webrtc: WebRTCConfig,
}

/// `WebRTC`-specific configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct WebRTCConfig {
    /// The 1to1 NAT ips.
    ///
    /// Useful when the host is behind a NAT.
    #[serde(default)]
    pub nat_1to1_ips: Vec<String>,

    /// The ICE servers.
    #[serde(default)]
    pub ice_servers: Vec<WebRTCIceServer>,
}

/// The `WebRTC` ICE servers.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebRTCIceServer {
    /// The ice server urls.
    pub urls: Vec<String>,

    /// The username, if one is required.
    #[serde(default)]
    pub username: String,

    /// The password (credential), if one is required.
    #[serde(default)]
    pub credential: String,
}

impl WebRTCConfig {
    fn default_ice_servers() -> Vec<WebRTCIceServer> {
        vec![WebRTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            username: "".to_string(),
            credential: "".to_string(),
        }]
    }
}

impl Default for WebRTCConfig {
    fn default() -> Self {
        Self {
            nat_1to1_ips: Vec::default(),
            ice_servers: Self::default_ice_servers(),
        }
    }
}

impl From<WebRTCIceServer> for RTCIceServer {
    fn from(server: WebRTCIceServer) -> Self {
        Self {
            urls: server.urls,
            username: server.username,
            credential: server.credential,
            credential_type: RTCIceCredentialType::Password,
        }
    }
}
