pub mod loader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub use loader::ConfigLoader;

/// Supported proxy types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Shadowsocks,
    #[serde(rename = "ss")]
    ShadowsocksShort,
    VMess,
    VLESS,
    Trojan,
    Hysteria,
    Hysteria2,
    #[serde(rename = "wireguard")]
    WireGuard,
    Socks5,
    #[serde(rename = "socks")]
    Socks,
    Http,
    Https,
    // Add support for newer/custom proxy types
    #[serde(rename = "anytls")]
    AnyTLS,
}

impl FromStr for ProxyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "shadowsocks" | "ss" => Ok(ProxyType::Shadowsocks),
            "vmess" => Ok(ProxyType::VMess),
            "vless" => Ok(ProxyType::VLESS),
            "trojan" => Ok(ProxyType::Trojan),
            "hysteria" => Ok(ProxyType::Hysteria),
            "hysteria2" => Ok(ProxyType::Hysteria2),
            "wireguard" | "wg" => Ok(ProxyType::WireGuard),
            "socks5" | "socks" => Ok(ProxyType::Socks5),
            "http" => Ok(ProxyType::Http),
            "https" => Ok(ProxyType::Https),
            "anytls" => Ok(ProxyType::AnyTLS),
            _ => Err(format!("Unknown proxy type: {s}")),
        }
    }
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Shadowsocks | ProxyType::ShadowsocksShort => write!(f, "Shadowsocks"),
            ProxyType::VMess => write!(f, "VMess"),
            ProxyType::VLESS => write!(f, "VLESS"),
            ProxyType::Trojan => write!(f, "Trojan"),
            ProxyType::Hysteria => write!(f, "Hysteria"),
            ProxyType::Hysteria2 => write!(f, "Hysteria2"),
            ProxyType::WireGuard => write!(f, "WireGuard"),
            ProxyType::Socks5 | ProxyType::Socks => write!(f, "SOCKS5"),
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Https => write!(f, "HTTPS"),
            ProxyType::AnyTLS => write!(f, "AnyTLS"),
        }
    }
}

/// Main proxy configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,
    pub server: String,
    #[serde(deserialize_with = "deserialize_port")]
    pub port: u16,
    #[serde(flatten)]
    pub config: ProxyParameters,
}

/// Proxy parameters that vary by protocol type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyParameters {
    // Common TLS settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    #[serde(rename = "skip-cert-verify", skip_serializing_if = "Option::is_none")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sni: Option<String>,

    // Authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    // Shadowsocks specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(rename = "plugin-opts", skip_serializing_if = "Option::is_none")]
    pub plugin_opts: Option<HashMap<String, serde_yaml::Value>>,

    // VMess/VLESS specific
    #[serde(rename = "alterId", skip_serializing_if = "Option::is_none")]
    pub alter_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,

    // Transport options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(rename = "ws-opts", skip_serializing_if = "Option::is_none")]
    pub ws_opts: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "grpc-opts", skip_serializing_if = "Option::is_none")]
    pub grpc_opts: Option<HashMap<String, serde_yaml::Value>>,
    #[serde(rename = "h2-opts", skip_serializing_if = "Option::is_none")]
    pub h2_opts: Option<HashMap<String, serde_yaml::Value>>,

    // Hysteria specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_number", skip_serializing_if = "Option::is_none")]
    pub up: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_number", skip_serializing_if = "Option::is_none")]
    pub down: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
    #[serde(rename = "auth-str", skip_serializing_if = "Option::is_none")]
    pub auth_str: Option<String>,
    #[serde(rename = "ca-str", skip_serializing_if = "Option::is_none")]
    pub ca_str: Option<String>,

    // Additional common fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tfo: Option<bool>,
    #[serde(rename = "client-fingerprint", skip_serializing_if = "Option::is_none")]
    pub client_fingerprint: Option<String>,

    // Hysteria2 specific fields (ports field for port ranges)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<String>,

    // Trojan/TLS specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    // Connection optimization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mptcp: Option<bool>,
    #[serde(rename = "ip-version", skip_serializing_if = "Option::is_none")]
    pub ip_version: Option<String>,
    #[serde(rename = "interface-name", skip_serializing_if = "Option::is_none")]
    pub interface_name: Option<String>,
    #[serde(rename = "routing-mark", skip_serializing_if = "Option::is_none")]
    pub routing_mark: Option<u32>,
    #[serde(rename = "dialer-proxy", skip_serializing_if = "Option::is_none")]
    pub dialer_proxy: Option<String>,

    // SMUX configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smux: Option<HashMap<String, serde_yaml::Value>>,

    // Catch-all for unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    #[serde(rename = "skip-cert-verify")]
    pub skip_cert_verify: bool,
    pub server_name: Option<String>,
    pub alpn: Option<Vec<String>>,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    pub path: Option<String>,
    pub host: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Root configuration structure for Clash config files
#[derive(Debug, Serialize, Deserialize)]
pub struct ClashConfig {
    pub proxies: Vec<ProxyConfig>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_yaml::Value>,
}

/// Custom deserializer for port field that can be either string or number
fn deserialize_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct PortVisitor;

    impl<'de> Visitor<'de> for PortVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a port number as string or integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.parse::<u16>()
                .map_err(|_| de::Error::custom(format!("invalid port string: {v}")))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v >= 0 && v <= u16::MAX as i64 {
                Ok(v as u16)
            } else {
                Err(de::Error::custom(format!("port out of range: {v}")))
            }
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v <= u16::MAX as u64 {
                Ok(v as u16)
            } else {
                Err(de::Error::custom(format!("port out of range: {v}")))
            }
        }
    }

    deserializer.deserialize_any(PortVisitor)
}

/// Custom deserializer for fields that can be either string or number
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}
