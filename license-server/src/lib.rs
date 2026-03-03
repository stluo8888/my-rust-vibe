use serde::{Serialize, Deserialize};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use base64::{engine::general_purpose, Engine as _};
use warp::Filter;
use std::sync::Arc;

#[derive(Deserialize, Serialize, Clone)]
pub struct Request {
    pub mid: String,
}

#[derive(Serialize)]
struct License {
    mid: String,
    sig: String,
}

/// load keypair from env
pub fn load_keypair() -> Keypair {
    let priv_b64 = std::env::var("LICENSE_PRIVATE_KEY").expect("LICENSE_PRIVATE_KEY not set");
    let priv_bytes = general_purpose::STANDARD.decode(priv_b64).expect("invalid base64");
    let secret = SecretKey::from_bytes(&priv_bytes).expect("invalid private key");
    let public = PublicKey::from(&secret);
    Keypair { secret, public }
}

/// build warp filter for issuing license
pub fn issue_filter(keypair: Arc<Keypair>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("issue"))
        .and(warp::body::json())
        .map(move |req: Request| {
            let sig = keypair.sign(req.mid.as_bytes());
            let lic = License {
                mid: req.mid,
                sig: general_purpose::STANDARD.encode(sig.to_bytes()),
            };
            warp::reply::json(&lic)
        })
}
