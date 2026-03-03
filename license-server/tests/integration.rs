use license_server::Request;
use ed25519_dalek::{Keypair, SecretKey, PublicKey};
use base64::{engine::general_purpose, Engine as _};
use std::sync::Arc;

#[tokio::test]
async fn issue_endpoint_returns_signed_license() {
    let priv_bytes = general_purpose::STANDARD.decode("vnIhyNlPDoa0/xkuWsb9+lcBn7oKK0XYKM0zuRhA/94=").unwrap();
    let secret = SecretKey::from_bytes(&priv_bytes).unwrap();
    let public = PublicKey::from(&secret);
    let keypair = Arc::new(Keypair { secret, public });

    let filter = license_server::issue_filter(keypair.clone());
    let req = Request { mid: "machineX".to_string() };
    let resp = warp::test::request()
        .method("POST")
        .path("/issue")
        .json(&req)
        .reply(&filter)
        .await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body["mid"], "machineX");
    assert!(body["sig"].as_str().unwrap().len() > 0);
}
