use std::sync::Arc;

use license_server::{load_keypair, issue_filter};

#[tokio::main]
async fn main() {
    let keypair = Arc::new(load_keypair());
    let issue = issue_filter(keypair.clone());

    println!("license server running on 0.0.0.0:3030, using ED25519 key");
    warp::serve(issue).run(([0, 0, 0, 0], 3030)).await;
}
