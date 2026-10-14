use anyhow::{Context, Result};
use openssl::pkey::{PKey, Private};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use openssl::x509::X509;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_openssl::SslStream;

use crate::net::framing::{read_delimited_message, write_delimited_message};
use crate::proto::remote::{
    RemoteAppLinkLaunchRequest, RemoteConfigure, RemoteDeviceInfo, RemoteDirection,
    RemoteEditInfo, RemoteImeBatchEdit, RemoteImeObject, RemoteKeyCode, RemoteKeyInject,
    RemoteMessage, RemotePingResponse, RemoteSetActive,
};

pub const REMOTE_PORT: u16 = 6466;

pub const FEATURE_PING: i32 = 1 << 0;
pub const FEATURE_KEY: i32 = 1 << 1;
pub const FEATURE_IME: i32 = 1 << 2;
pub const FEATURE_POWER: i32 = 1 << 5;
pub const FEATURE_VOLUME: i32 = 1 << 6;
pub const FEATURE_APP_LINK: i32 = 1 << 9;

pub const ACTIVE_FEATURES: i32 =
    FEATURE_PING | FEATURE_KEY | FEATURE_IME | FEATURE_POWER | FEATURE_VOLUME | FEATURE_APP_LINK;

#[derive(Debug, Clone, Default)]
pub struct DeviceDetails {
    pub vendor: String,
    pub model: String,
    pub app_version: String,
}

#[derive(Debug, Clone)]
pub enum TvEvent {
    Connected(DeviceDetails),
    Disconnected(String),
    PowerStateChanged(bool),
    VolumeChanged {
        level: u32,
        max: u32,
        muted: bool,
    },
    ActiveAppChanged(String),
    ProtocolLog(String),
}

#[derive(Debug, Clone)]
pub enum TvCommand {
    Key {
        code: RemoteKeyCode,
        direction: RemoteDirection,
    },
    Text(String),
    LaunchApp(String),
    Disconnect,
}

#[derive(Clone)]
pub struct TvClientHandle {
    cmd_tx: mpsc::Sender<TvCommand>,
}

impl TvClientHandle {
    pub async fn send_key(&self, code: RemoteKeyCode, direction: RemoteDirection) -> Result<()> {
        self.cmd_tx
            .send(TvCommand::Key { code, direction })
            .await
            .context("Client task closed")
    }

    pub async fn send_text(&self, text: String) -> Result<()> {
        self.cmd_tx
            .send(TvCommand::Text(text))
            .await
            .context("Client task closed")
    }

    pub async fn launch_app(&self, app_link: String) -> Result<()> {
        self.cmd_tx
            .send(TvCommand::LaunchApp(app_link))
            .await
            .context("Client task closed")
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.cmd_tx
            .send(TvCommand::Disconnect)
            .await
            .context("Client task closed")
    }
}

pub struct TvConnection;

impl TvConnection {
    /// Connects to Google TV on port 6466 and runs the background control loop.
    pub async fn connect(
        target: SocketAddr,
        client_cert: X509,
        client_key: PKey<Private>,
        event_tx: mpsc::Sender<TvEvent>,
    ) -> Result<TvClientHandle> {
        let mut connector_builder =
            SslConnector::builder(SslMethod::tls()).context("Failed to create SSL connector")?;
        connector_builder
            .set_certificate(&client_cert)
            .context("Failed to set client certificate")?;
        connector_builder
            .set_private_key(&client_key)
            .context("Failed to set client key")?;
        connector_builder.set_verify(SslVerifyMode::NONE);
        let connector = connector_builder.build();

        let tcp = TcpStream::connect(target)
            .await
            .with_context(|| format!("Failed to connect to TV at {}", target))?;

        let ssl = connector
            .configure()
            .context("Failed to configure SSL")?
            .into_ssl("atvremote")
            .context("Failed into_ssl")?;

        let mut stream = SslStream::new(ssl, tcp).context("Failed into SslStream")?;
        Pin::new(&mut stream)
            .connect()
            .await
            .context("TLS handshake failed with TV remote port")?;

        let (cmd_tx, mut cmd_rx) = mpsc::channel::<TvCommand>(32);

        tokio::spawn(async move {
            let mut ime_counter = 0i32;
            let mut field_counter = 0i32;
            let mut device_info = DeviceDetails::default();

            loop {
                tokio::select! {
                    // Check for incoming commands to send to TV
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(TvCommand::Key { code, direction }) => {
                                let mut msg = RemoteMessage::default();
                                msg.remote_key_inject = Some(RemoteKeyInject {
                                    key_code: code as i32,
                                    direction: direction as i32,
                                });
                                let _ = event_tx.send(TvEvent::ProtocolLog(format!("TX KeyInject: {:?} ({:?})", code, direction))).await;
                                if let Err(e) = write_delimited_message(&mut stream, &msg).await {
                                    let _ = event_tx.send(TvEvent::Disconnected(format!("Write error: {}", e))).await;
                                    break;
                                }
                            }
                            Some(TvCommand::Text(text)) => {
                                let len = (text.chars().count().saturating_sub(1)) as i32;
                                let ime_obj = RemoteImeObject {
                                    start: len,
                                    end: len,
                                    value: text.clone(),
                                };
                                let edit = RemoteEditInfo {
                                    insert: 1,
                                    text_field_status: Some(ime_obj),
                                };
                                let batch = RemoteImeBatchEdit {
                                    ime_counter,
                                    field_counter,
                                    edit_info: vec![edit],
                                };
                                let mut msg = RemoteMessage::default();
                                msg.remote_ime_batch_edit = Some(batch);
                                let _ = event_tx.send(TvEvent::ProtocolLog(format!("TX IME text: '{}'", text))).await;
                                if let Err(e) = write_delimited_message(&mut stream, &msg).await {
                                    let _ = event_tx.send(TvEvent::Disconnected(format!("Write error: {}", e))).await;
                                    break;
                                }
                            }
                            Some(TvCommand::LaunchApp(app_link)) => {
                                let mut msg = RemoteMessage::default();
                                msg.remote_app_link_launch_request = Some(RemoteAppLinkLaunchRequest {
                                    app_link: app_link.clone(),
                                });
                                let _ = event_tx.send(TvEvent::ProtocolLog(format!("TX LaunchApp: {}", app_link))).await;
                                if let Err(e) = write_delimited_message(&mut stream, &msg).await {
                                    let _ = event_tx.send(TvEvent::Disconnected(format!("Write error: {}", e))).await;
                                    break;
                                }
                            }
                            Some(TvCommand::Disconnect) | None => {
                                let _ = event_tx.send(TvEvent::Disconnected("Disconnected by user".to_string())).await;
                                break;
                            }
                        }
                    }

                    // Check for incoming protobuf messages from TV
                    res = read_delimited_message::<_, RemoteMessage>(&mut stream) => {
                        match res {
                            Ok(msg) => {
                                // 1. Ping Request -> Send Ping Response
                                if let Some(ping) = msg.remote_ping_request {
                                    let mut reply = RemoteMessage::default();
                                    reply.remote_ping_response = Some(RemotePingResponse {
                                        val1: ping.val1,
                                    });
                                    if let Err(e) = write_delimited_message(&mut stream, &reply).await {
                                        let _ = event_tx.send(TvEvent::Disconnected(format!("Ping reply error: {}", e))).await;
                                        break;
                                    }
                                }
                                // 2. Configuration Handshake
                                else if let Some(cfg) = msg.remote_configure {
                                    if let Some(dev) = cfg.device_info {
                                        device_info = DeviceDetails {
                                            vendor: dev.vendor,
                                            model: dev.model,
                                            app_version: dev.app_version,
                                        };
                                        let _ = event_tx.send(TvEvent::Connected(device_info.clone())).await;
                                    }
                                    // Reply with client configure
                                    let mut reply = RemoteMessage::default();
                                    let mut client_cfg = RemoteConfigure::default();
                                    client_cfg.code1 = ACTIVE_FEATURES;
                                    client_cfg.device_info = Some(RemoteDeviceInfo {
                                        unknown1: 1,
                                        unknown2: "1".into(),
                                        package_name: "atvremote".into(),
                                        app_version: "1.0.0".into(),
                                        ..Default::default()
                                    });
                                    reply.remote_configure = Some(client_cfg);
                                    let _ = event_tx.send(TvEvent::ProtocolLog("RX RemoteConfigure -> TX ClientConfigure".to_string())).await;
                                    if let Err(e) = write_delimited_message(&mut stream, &reply).await {
                                        let _ = event_tx.send(TvEvent::Disconnected(format!("Configure reply error: {}", e))).await;
                                        break;
                                    }
                                }
                                // 3. Set Active
                                else if msg.remote_set_active.is_some() {
                                    let mut reply = RemoteMessage::default();
                                    reply.remote_set_active = Some(RemoteSetActive {
                                        active: ACTIVE_FEATURES,
                                    });
                                    let _ = event_tx.send(TvEvent::ProtocolLog("RX RemoteSetActive -> TX ClientSetActive".to_string())).await;
                                    if let Err(e) = write_delimited_message(&mut stream, &reply).await {
                                        let _ = event_tx.send(TvEvent::Disconnected(format!("SetActive reply error: {}", e))).await;
                                        break;
                                    }
                                }
                                // 4. Power Status
                                else if let Some(start) = msg.remote_start {
                                    let _ = event_tx.send(TvEvent::PowerStateChanged(start.started)).await;
                                    let _ = event_tx.send(TvEvent::ProtocolLog(format!("RX Power State: {}", if start.started { "ON" } else { "OFF/STANDBY" }))).await;
                                }
                                // 5. Volume Status
                                else if let Some(vol) = msg.remote_set_volume_level {
                                    let _ = event_tx.send(TvEvent::VolumeChanged {
                                        level: vol.volume_level,
                                        max: vol.volume_max,
                                        muted: vol.volume_muted,
                                    }).await;
                                    let _ = event_tx.send(TvEvent::ProtocolLog(format!("RX Volume: {}/{} (Muted: {})", vol.volume_level, vol.volume_max, vol.volume_muted))).await;
                                }
                                // 6. Active App Status
                                else if let Some(ime) = msg.remote_ime_key_inject {
                                    if let Some(app) = ime.app_info {
                                        let _ = event_tx.send(TvEvent::ActiveAppChanged(app.app_package.clone())).await;
                                        let _ = event_tx.send(TvEvent::ProtocolLog(format!("RX Active App: {}", app.app_package))).await;
                                    }
                                }
                                // 7. IME Batch Counters
                                else if let Some(ime) = msg.remote_ime_batch_edit {
                                    ime_counter = ime.ime_counter;
                                    field_counter = ime.field_counter;
                                }
                                // 8. Remote Error
                                else if let Some(err) = msg.remote_error {
                                    let _ = event_tx.send(TvEvent::ProtocolLog(format!("RX Error from TV: value={}", err.value))).await;
                                }
                            }
                            Err(e) => {
                                let _ = event_tx.send(TvEvent::Disconnected(format!("Connection closed or error: {}", e))).await;
                                break;
                            }
                        }
                    }

                    // Watchdog: detect if connection idle for > 20s
                    _ = tokio::time::sleep(Duration::from_secs(25)) => {
                        let _ = event_tx.send(TvEvent::Disconnected("Connection timed out (no ping from TV for 25s)".to_string())).await;
                        break;
                    }
                }
            }
        });

        Ok(TvClientHandle { cmd_tx })
    }
}
