use serde::{Serialize, Deserialize};
use ed25519_dalek::{PublicKey, SecretKey, Keypair, Signature, Signer, Verifier};
use base64::{engine::general_purpose, Engine as _};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct License {
    pub mid: String,
    pub sig: String,
}

// 默认公钥, 也可通过环境变量覆盖
const PUBLIC_KEY_B64: &str = "VCDP6ZGvHvoalvbDB6sR/QRvR0Vr4x7iEuWhdnMJr4w=";

/// 验证 license 文件是否由内置公钥签名并且包含正确的 mid
pub fn verify_license(mid: &str, path: &str) -> bool {
    let pub_b64 = std::env::var("LICENSE_PUBLIC_KEY").unwrap_or_else(|_| PUBLIC_KEY_B64.to_string());
    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(lic) = serde_json::from_str::<License>(&contents) {
            if let (Ok(pub_bytes), Ok(sig_bytes)) = (
                general_purpose::STANDARD.decode(pub_b64),
                general_purpose::STANDARD.decode(&lic.sig),
            ) {
                if let (Ok(pubkey), Ok(sig)) = (
                    PublicKey::from_bytes(&pub_bytes),
                    Signature::from_bytes(&sig_bytes),
                ) {
                    if pubkey.verify(lic.mid.as_bytes(), &sig).is_ok() && lic.mid == mid {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// 用给定私钥对 mid 签名，返回 License 对象
pub fn make_license(mid: &str, private_key_b64: &str) -> License {
    let priv_bytes = general_purpose::STANDARD.decode(private_key_b64).expect("invalid base64");
    let secret = SecretKey::from_bytes(&priv_bytes).expect("invalid key");
    let public = PublicKey::from(&secret);
    let pair = Keypair { secret, public };
    let sig = pair.sign(mid.as_bytes());
    License { mid: mid.to_string(), sig: general_purpose::STANDARD.encode(sig.to_bytes()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    // sample private key matching PUBLIC_KEY_B64 above
    const TEST_PRIV: &str = "vnIhyNlPDoa0/xkuWsb9+lcBn7oKK0XYKM0zuRhA/94=";

    #[test]
    fn valid_license_passes() {
        let mid = "machine123";
        let lic = make_license(mid, TEST_PRIV);
        let json = serde_json::to_string(&lic).unwrap();
        let tmp = NamedTempFile::new().unwrap();
        fs::write(tmp.path(), &json).unwrap();
        assert!(verify_license(mid, tmp.path().to_str().unwrap()));
    }

    #[test]
    fn invalid_signature_fails() {
        let mid = "machine123";
        let mut lic = make_license(mid, TEST_PRIV);
        // modify signature
        lic.sig = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string();
        let json = serde_json::to_string(&lic).unwrap();
        let tmp = NamedTempFile::new().unwrap();
        fs::write(tmp.path(), &json).unwrap();
        assert!(!verify_license(mid, tmp.path().to_str().unwrap()));
    }

    #[test]
    fn wrong_mid_fails() {
        let lic = make_license("machineA", TEST_PRIV);
        let json = serde_json::to_string(&lic).unwrap();
        let tmp = NamedTempFile::new().unwrap();
        fs::write(tmp.path(), &json).unwrap();
        assert!(!verify_license("different", tmp.path().to_str().unwrap()));
    }

    #[test]
    fn missing_file_fails() {
        assert!(!verify_license("anything", "nonexistent.txt"));
    }
}
