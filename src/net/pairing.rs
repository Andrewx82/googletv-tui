use anyhow::{bail, Context, Result};
use openssl::pkey::{PKey, Private};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use openssl::x509::X509;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

use crate::net::cert::extract_modulus_and_exponent;
use crate::net::framing::{read_delimited_message, write_delimited_message};
use crate::proto::polo::{
    options::encoding::EncodingType, options::Encoding, options::RoleType,
    outer_message::Status, Configuration, Options, OuterMessage, PairingRequest, Secret,
};

pub const PAIRING_PORT: u16 = 6467;

/// Represents an ongoing pairing handshake with Google TV
pub struct PairingSession {
    stream: SslStream<TcpStream>,
    client_cert: X509,
    server_cert: X509,
    pub server_name: String,
}

impl PairingSession {
    /// Connects to Google TV on port 6467 and starts the pairing negotiation.
    /// Once this future resolves, the TV is showing the 6-character PIN code on its screen.
    pub async fn start(
        target: SocketAddr,
        client_name: &str,
        client_cert: X509,
        client_key: PKey<Private>,
    ) -> Result<Self> {
        let mut connector_builder =
            SslConnector::builder(SslMethod::tls()).context("Failed to create SSL connector builder")?;
        connector_builder
            .set_certificate(&client_cert)
            .context("Failed to set client certificate")?;
        connector_builder
            .set_private_key(&client_key)
            .context("Failed to set client private key")?;
        connector_builder.set_verify(SslVerifyMode::NONE);
        let connector = connector_builder.build();

        let tcp = TcpStream::connect(target)
            .await
            .with_context(|| format!("Failed to connect to pairing port at {}", target))?;

        let ssl = connector
            .configure()
            .context("Failed to configure SSL")?
            .into_ssl("atvremote")
            .context("Failed into_ssl")?;

        let mut stream = SslStream::new(ssl, tcp).context("Failed to wrap SslStream")?;
        Pin::new(&mut stream)
            .connect()
            .await
            .context("TLS handshake failed with TV during pairing")?;

        let server_cert = stream
            .ssl()
            .peer_certificate()
            .ok_or_else(|| anyhow::anyhow!("TV did not send its TLS certificate"))?;

        // 1. Send PairingRequest
        let req = OuterMessage {
            protocol_version: 2,
            status: Status::Ok as i32,
            pairing_request: Some(PairingRequest {
                service_name: "atvremote".to_string(),
                client_name: Some(client_name.to_string()),
            }),
            ..Default::default()
        };
        write_delimited_message(&mut stream, &req)
            .await
            .context("Failed sending pairing request")?;

        // 2. Expect PairingRequestAck
        let ack: OuterMessage = read_delimited_message(&mut stream)
            .await
            .context("Failed reading pairing request ack")?;
        if ack.status != Status::Ok as i32 {
            bail!("Pairing request rejected by device: status {}", ack.status);
        }
        let server_name = ack
            .pairing_request_ack
            .and_then(|a| a.server_name)
            .unwrap_or_else(|| "Google TV".to_string());

        // 3. Send Options
        let opts = OuterMessage {
            protocol_version: 2,
            status: Status::Ok as i32,
            options: Some(Options {
                preferred_role: Some(RoleType::Input as i32),
                input_encodings: vec![Encoding {
                    r#type: EncodingType::Hexadecimal as i32,
                    symbol_length: 6,
                }],
                output_encodings: vec![],
            }),
            ..Default::default()
        };
        write_delimited_message(&mut stream, &opts)
            .await
            .context("Failed sending pairing options")?;

        // 4. Expect Options from TV
        let server_opts: OuterMessage = read_delimited_message(&mut stream)
            .await
            .context("Failed reading TV options")?;
        if server_opts.status != Status::Ok as i32 {
            bail!("Pairing options rejected: status {}", server_opts.status);
        }

        // 5. Send Configuration
        let cfg = OuterMessage {
            protocol_version: 2,
            status: Status::Ok as i32,
            configuration: Some(Configuration {
                client_role: RoleType::Input as i32,
                encoding: Encoding {
                    r#type: EncodingType::Hexadecimal as i32,
                    symbol_length: 6,
                },
            }),
            ..Default::default()
        };
        write_delimited_message(&mut stream, &cfg)
            .await
            .context("Failed sending pairing configuration")?;

        // 6. Expect ConfigurationAck (PIN is now displayed on TV!)
        let cfg_ack: OuterMessage = read_delimited_message(&mut stream)
            .await
            .context("Failed reading configuration ack")?;
        if cfg_ack.status != Status::Ok as i32 {
            bail!("Pairing configuration rejected: status {}", cfg_ack.status);
        }

        Ok(Self {
            stream,
            client_cert,
            server_cert,
            server_name,
        })
    }

    /// Submits the 6-character hex PIN displayed on TV to complete the pairing.
    pub async fn finish(mut self, pin_raw: &str) -> Result<()> {
        let pin = pin_raw.trim().to_uppercase();
        if pin.len() != 6 {
            bail!("Pairing code must be exactly 6 hex characters (e.g. 1A2B3C)");
        }
        if !pin.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("Pairing code must contain valid hexadecimal characters (0-9, A-F)");
        }

        let (client_n, client_e) = extract_modulus_and_exponent(&self.client_cert)
            .context("Failed to extract client modulus/exponent")?;
        let (server_n, server_e) = extract_modulus_and_exponent(&self.server_cert)
            .context("Failed to extract server modulus/exponent")?;

        let mut hasher = Sha256::new();
        hasher.update(hex::decode(&client_n).context("Invalid client n hex")?);
        hasher.update(hex::decode(&client_e).context("Invalid client e hex")?);
        hasher.update(hex::decode(&server_n).context("Invalid server n hex")?);
        hasher.update(hex::decode(&server_e).context("Invalid server e hex")?);
        hasher.update(hex::decode(&pin[2..]).context("Invalid pin remainder hex")?);
        let digest = hasher.finalize();

        let expected_first_byte =
            u8::from_str_radix(&pin[0..2], 16).context("Invalid first pin byte")?;
        if digest[0] != expected_first_byte {
            bail!(
                "Secret hash verification failed (computed {:02X}, expected {:02X}). Check that PIN was entered correctly.",
                digest[0],
                expected_first_byte
            );
        }

        let secret_msg = OuterMessage {
            protocol_version: 2,
            status: Status::Ok as i32,
            secret: Some(Secret {
                secret: digest.to_vec(),
            }),
            ..Default::default()
        };
        write_delimited_message(&mut self.stream, &secret_msg)
            .await
            .context("Failed sending pairing secret")?;

        let secret_ack: OuterMessage = read_delimited_message(&mut self.stream)
            .await
            .context("Failed reading secret ack")?;
        if secret_ack.status != Status::Ok as i32 {
            bail!("TV rejected pairing secret: status {}", secret_ack.status);
        }

        Ok(())
    }
}
