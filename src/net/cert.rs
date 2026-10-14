use anyhow::{Context, Result};
use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, SubjectAlternativeName};
use openssl::x509::{X509NameBuilder, X509};
use std::fs;
use std::path::{Path, PathBuf};

pub struct CertManager {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl CertManager {
    pub fn new(base_dir: &Path) -> Self {
        Self {
            cert_path: base_dir.join("cert.pem"),
            key_path: base_dir.join("key.pem"),
        }
    }

    /// Loads existing certificate and private key, or generates a new pair and saves them.
    pub fn load_or_generate(&self, common_name: &str) -> Result<(X509, PKey<Private>)> {
        if self.cert_path.exists() && self.key_path.exists() {
            let cert_bytes = fs::read(&self.cert_path)
                .context("Failed to read client certificate from disk")?;
            let key_bytes =
                fs::read(&self.key_path).context("Failed to read client private key from disk")?;

            let cert = X509::from_pem(&cert_bytes).context("Failed to parse X509 PEM cert")?;
            let key =
                PKey::private_key_from_pem(&key_bytes).context("Failed to parse private key PEM")?;
            return Ok((cert, key));
        }

        let (cert, key) = generate_self_signed_cert(common_name)?;

        if let Some(parent) = self.cert_path.parent() {
            fs::create_dir_all(parent).context("Failed to create certificate directory")?;
        }

        let cert_pem = cert.to_pem().context("Failed to encode cert to PEM")?;
        let key_pem = key
            .private_key_to_pem_pkcs8()
            .context("Failed to encode private key to PEM")?;

        fs::write(&self.cert_path, cert_pem).context("Failed to write certificate file")?;
        fs::write(&self.key_path, key_pem).context("Failed to write private key file")?;

        Ok((cert, key))
    }
}

/// Generates a 2048-bit RSA self-signed certificate matching Google TV expectations
pub fn generate_self_signed_cert(common_name: &str) -> Result<(X509, PKey<Private>)> {
    let rsa = Rsa::generate(2048).context("Failed to generate 2048-bit RSA key")?;
    let pkey = PKey::from_rsa(rsa).context("Failed to create PKey from RSA")?;

    let mut name = X509NameBuilder::new().context("Failed to create X509 name builder")?;
    name.append_entry_by_text("CN", common_name)
        .context("Failed to set CN")?;
    let name = name.build();

    let mut builder = X509::builder().context("Failed to create X509 builder")?;
    builder.set_version(2)?; // X509 v3

    let bn = BigNum::from_u32(1000).context("Failed to create BigNum for serial")?;
    let serial = Asn1Integer::from_bn(&bn).context("Failed to create Asn1Integer serial")?;
    builder.set_serial_number(&serial)?;
    builder.set_subject_name(&name)?;
    builder.set_issuer_name(&name)?;
    builder.set_pubkey(&pkey)?;

    let nb = Asn1Time::days_from_now(0).context("Failed to set not_before")?;
    let na = Asn1Time::days_from_now(3650).context("Failed to set not_after")?;
    builder.set_not_before(&nb)?;
    builder.set_not_after(&na)?;

    let bc = BasicConstraints::new().ca().pathlen(0).build()?;
    builder.append_extension(bc)?;

    let ctx = builder.x509v3_context(None, None);
    let mut san = SubjectAlternativeName::new();
    san.dns(common_name);
    let san_ext = san.build(&ctx)?;
    builder.append_extension(san_ext)?;

    builder.sign(&pkey, MessageDigest::sha256())?;
    let cert = builder.build();

    Ok((cert, pkey))
}

/// Extracts RSA modulus and exponent as uppercase hexadecimal strings,
/// padded to even length for byte alignment.
pub fn extract_modulus_and_exponent(cert: &X509) -> Result<(String, String)> {
    let pubkey = cert.public_key().context("Failed to get public key")?;
    let rsa = pubkey.rsa().context("Public key is not RSA")?;

    let n_hex = rsa.n().to_hex_str()?.to_string();
    let e_hex = rsa.e().to_hex_str()?.to_string();

    let n_padded = if n_hex.len() % 2 != 0 {
        format!("0{}", n_hex).to_uppercase()
    } else {
        n_hex.to_uppercase()
    };

    let e_padded = if e_hex.len() % 2 != 0 {
        format!("0{}", e_hex).to_uppercase()
    } else {
        e_hex.to_uppercase()
    };

    Ok((n_padded, e_padded))
}
