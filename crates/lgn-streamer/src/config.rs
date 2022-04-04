use std::fmt::Display;

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

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"
Encoding:
- enable_hw_encoding: {}

WebRTC:
{}"#,
            self.enable_hw_encoding, self.webrtc,
        )
    }
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

impl Display for WebRTCConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.nat_1to1_ips.is_empty() {
            writeln!(f, "- NAT 1-to-1 IPs: none")
        } else {
            writeln!(f, "- NAT 1-to-1 IPs: {}", self.nat_1to1_ips.join(", "))
        }?;

        if self.ice_servers.is_empty() {
            writeln!(f, "- ICE servers: none")
        } else {
            write!(
                f,
                r#"- ICE servers:
{}"#,
                self.ice_servers
                    .iter()
                    .map(|server| format!("  - {}", server))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }
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

impl Display for WebRTCIceServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.username.is_empty() {
            write!(f, "{} (no authentication)", self.urls.join(", "),)
        } else {
            write!(
                f,
                "{} (username: {}, credential: <redacted>)",
                self.urls.join(", "),
                self.username
            )
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
