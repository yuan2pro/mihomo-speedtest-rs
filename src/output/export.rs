use crate::Result;
use crate::config::{ClashConfig, ProxyConfig};
use crate::core::SpeedTestResult;
use std::collections::HashMap;
use std::path::Path;

/// Exporter for configuration files
pub struct ConfigExporter;

impl ConfigExporter {
    /// Export successful proxies to a Clash config file
    pub async fn export_clash_config<P: AsRef<Path>>(
        results: &[SpeedTestResult],
        output_path: P,
    ) -> Result<()> {
 
        // Filter original proxy configs to keep only successful ones
        let successful_proxies: Vec<ProxyConfig> = results
            .iter()
            .filter(|r| r.is_successful())
            .map(|r| r.proxy_config.clone())
            .collect();

        // Create Clash config structure
        let config = ClashConfig {
            proxies: successful_proxies,
            other: HashMap::new(),
        };

        // Serialize to YAML
        let yaml_content = serde_yaml::to_string(&config)?;

        // Write to file
        tokio::fs::write(output_path, yaml_content).await?;

        Ok(())
    }

    /// Export results as JSON
    pub async fn export_json<P: AsRef<Path>>(
        results: &[SpeedTestResult],
        output_path: P,
    ) -> Result<()> {
        let json_content = serde_json::to_string_pretty(results)?;
        tokio::fs::write(output_path, json_content).await?;
        Ok(())
    }

    /// Generate renamed proxies with speed and location info
    pub fn rename_proxies_with_stats(
        results: &[SpeedTestResult],
    ) -> Vec<SpeedTestResult> {
        results
            .iter()
            .map(|result| {
                if result.is_successful() {
                    let mut renamed_proxy = result.proxy_config.clone();
                    renamed_proxy.name = Self::generate_new_name(&result.proxy_config, result);
                    SpeedTestResult {
                        proxy_config: renamed_proxy,
                        ..result.clone()
                    }
                } else {
                    result.clone()
                }
            })
            .collect()
    }

    /// Generate a new proxy name with stats
    fn generate_new_name(proxy: &ProxyConfig, result: &SpeedTestResult) -> String {
        let speed_mbps = result.download_speed / (1024.0 * 1024.0);
        let latency_ms = result.latency.map_or(0, |l| l.as_millis());

        // Try to extract location from original name or use server
        let location = Self::extract_location(&proxy.name)
            .unwrap_or_else(|| Self::guess_location_from_server(&proxy.server));
        let proxy_name = proxy.name.clone();
        format!("{location} | {proxy_name} | 📈 {speed_mbps:.1}MB/s | ⏱️ {latency_ms}ms")
    }

    /// Extract location from proxy name
    fn extract_location(name: &str) -> Option<String> {
        // Simple heuristics to extract location
        let name_lower = name.to_lowercase();

        // Common location patterns
        let patterns = [
            ("hong kong", "🇭🇰 Hong Kong"),
            ("hk", "🇭🇰 Hong Kong"),
            ("singapore", "🇸🇬 Singapore"),
            ("japan", "🇯🇵 Japan"),
            ("jp", "🇯🇵 Japan"),
            ("tokyo", "🇯🇵 Tokyo"),
            ("united states", "🇺🇸 United States"),
            ("usa", "🇺🇸 USA"),
            ("us", "🇺🇸 USA"),
            ("canada", "🇨🇦 Canada"),
            ("germany", "🇩🇪 Germany"),
            ("de", "🇩🇪 Germany"),
            ("united kingdom", "🇬🇧 United Kingdom"),
            ("uk", "🇬🇧 UK"),
            ("france", "🇫🇷 France"),
            ("fr", "🇫🇷 France"),
            ("netherlands", "🇳🇱 Netherlands"),
            ("nl", "🇳🇱 Netherlands"),
            ("russia", "🇷🇺 Russia"),
            ("ru", "🇷🇺 Russia"),
            ("korea", "🇰🇷 Korea"),
            ("kr", "🇰🇷 Korea"),
            ("taiwan", "🇹🇼 Taiwan"),
            ("tw", "🇹🇼 Taiwan"),
        ];

        for (pattern, flag) in &patterns {
            if name_lower.contains(pattern) {
                return Some(flag.to_string());
            }
        }

        // If no pattern matches, try to use the original name
        if name.len() < 50 {
            Some(name.to_string())
        } else {
            None
        }
    }

    /// Guess location from server hostname/IP
    fn guess_location_from_server(server: &str) -> String {
        // Simple TLD-based location guessing
        if server.ends_with(".jp") || server.contains("japan") {
            "🇯🇵 Japan".to_string()
        } else if server.ends_with(".hk") || server.contains("hongkong") {
            "🇭🇰 Hong Kong".to_string()
        } else if server.ends_with(".sg") || server.contains("singapore") {
            "🇸🇬 Singapore".to_string()
        } else if server.ends_with(".us") || server.contains("usa") {
            "🇺🇸 USA".to_string()
        } else if server.ends_with(".de") || server.contains("germany") {
            "🇩🇪 Germany".to_string()
        } else if server.ends_with(".uk") || server.contains("britain") {
            "🇬🇧 UK".to_string()
        } else {
            format!("🌐 {}", server.split('.').next().unwrap_or(server))
        }
    }
}
