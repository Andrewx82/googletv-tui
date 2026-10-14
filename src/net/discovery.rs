use anyhow::{Context, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::Duration;
use tokio::sync::mpsc;

pub const SERVICE_TYPE: &str = "_androidtvremote2._tcp.local.";
pub const FALLBACK_SERVICE_TYPE: &str = "_googlecast._tcp.local.";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiscoveredDevice {
    pub name: String,
    pub ip: IpAddr,
    pub port: u16,
    pub service_type: String,
}

pub struct DiscoveryManager;

impl DiscoveryManager {
    /// Starts background discovery and streams discovered devices to `tx`.
    pub async fn start_discovery(
        tx: mpsc::Sender<DiscoveredDevice>,
        duration: Duration,
    ) -> Result<()> {
        let mdns = ServiceDaemon::new().context("Failed to create mDNS daemon")?;
        let receiver1 = mdns
            .browse(SERVICE_TYPE)
            .context("Failed to browse androidtvremote2 service")?;
        let receiver2 = mdns
            .browse(FALLBACK_SERVICE_TYPE)
            .context("Failed to browse googlecast service")?;

        tokio::spawn(async move {
            let mut seen = HashSet::new();
            let start = tokio::time::Instant::now();

            loop {
                if start.elapsed() > duration {
                    break;
                }

                tokio::select! {
                    Ok(event) = receiver1.recv_async() => {
                        if let ServiceEvent::ServiceResolved(info) = event {
                            let name = info.get_fullname().split('.').next().unwrap_or("Google TV").to_string();
                            for ip in info.get_addresses_v4() {
                                let dev = DiscoveredDevice {
                                    name: name.clone(),
                                    ip: IpAddr::V4(*ip),
                                    port: info.get_port(),
                                    service_type: SERVICE_TYPE.to_string(),
                                };
                                if seen.insert(dev.clone()) {
                                    let _ = tx.send(dev).await;
                                }
                            }
                        }
                    }
                    Ok(event) = receiver2.recv_async() => {
                        if let ServiceEvent::ServiceResolved(info) = event {
                            let name = info.get_fullname().split('.').next().unwrap_or("Chromecast").to_string();
                            for ip in info.get_addresses_v4() {
                                let dev = DiscoveredDevice {
                                    name: name.clone(),
                                    ip: IpAddr::V4(*ip),
                                    port: 6466, // Default remote port
                                    service_type: FALLBACK_SERVICE_TYPE.to_string(),
                                };
                                if seen.insert(dev.clone()) {
                                    let _ = tx.send(dev).await;
                                }
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                }
            }
        });

        Ok(())
    }
}
