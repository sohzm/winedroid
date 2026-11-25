use std::path::Path;

use apksig::apk_verifier;
use sha2::{Digest, Sha256};

use crate::error::{Result, WinedroidError};

pub fn extract_signing_certificates(apk_path: &Path) -> Result<Vec<Vec<u8>>> {
    // cite: frameworks-base ApkSignatureVerifier.java#unsafeGetCertsWithoutVerification, verify
    let verifier = apk_verifier::Builder::from_file(apk_path).build();
    match verifier.verify() {
        Ok(result) => {
            if result.is_verified() {
                Ok(result.get_signer_certificates().to_vec())
            } else {
                let errors = result
                    .get_all_errors()
                    .iter()
                    .map(|issue| issue.to_string())
                    .collect::<Vec<_>>();
                if errors.is_empty() {
                    Ok(Vec::new())
                } else {
                    Err(WinedroidError::new(format!(
                        "APK signature verification failed for `{}`: {}",
                        apk_path.display(),
                        errors.join("; ")
                    )))
                }
            }
        }
        Err(error) => {
            let message = error.to_string();
            if message.contains("No certificates")
                || message.contains("Not signed")
                || message.contains("No JAR signers")
                || message.contains("No signers")
            {
                Ok(Vec::new())
            } else {
                Err(WinedroidError::new(format!(
                    "failed to inspect APK signatures in `{}`: {message}",
                    apk_path.display()
                )))
            }
        }
    }
}

pub fn sha256_fingerprint_colon_hex(der: &[u8]) -> String {
    let digest = Sha256::digest(der);
    let mut out = String::with_capacity(digest.len() * 3);
    for (index, byte) in digest.iter().enumerate() {
        if index > 0 {
            out.push(':');
        }
        out.push_str(&format!("{byte:02X}"));
    }
    out
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

pub fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    let trimmed = hex.trim();
    if !trimmed.len().is_multiple_of(2) {
        return Err(WinedroidError::new("hex string must have an even length"));
    }
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    for chunk in trimmed.as_bytes().chunks_exact(2) {
        let pair = std::str::from_utf8(chunk)
            .map_err(|_| WinedroidError::new("hex string must be valid UTF-8"))?;
        let byte = u8::from_str_radix(pair, 16)
            .map_err(|_| WinedroidError::new(format!("invalid hex byte `{pair}`")))?;
        out.push(byte);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn hex_roundtrip_and_sha256_fingerprint_format() {
        let sample = [0x30u8, 0x82, 0x01, 0x0a];
        let encoded = hex_encode(&sample);
        assert_eq!(encoded, "3082010a");
        assert_eq!(hex_decode(&encoded).unwrap(), sample);

        let fp = sha256_fingerprint_colon_hex(&sample);
        assert_eq!(fp.len(), 32 * 2 + 31);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit() || c == ':'));
    }

    #[test]
    fn unsigned_zip_returns_empty_certificates_without_faking() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("winedroid-unsigned-{stamp}.apk"));
        {
            let file = std::fs::File::create(&path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("AndroidManifest.xml", options).unwrap();
            zip.write_all(b"dummy").unwrap();
            zip.finish().unwrap();
        }

        let result = extract_signing_certificates(&path);
        let _ = std::fs::remove_file(&path);
        match result {
            Ok(certs) => assert!(certs.is_empty()),
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    msg.contains("signature") || msg.contains("signer") || msg.contains("Manifest"),
                    "unexpected error for unsigned zip: {msg}"
                );
            }
        }
    }
}
