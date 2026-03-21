// Copyright © 2026 OBINNA JAMES EJIOFOR
// All Rights Reserved.
//
// TLBSS Demo Dashboard
// Web interface for interactive demo visualization

use axum::{routing::get, Json, Router};
use m_v_r_esprint1::demo_pipeline::{run_full_demo, DemoResult, MarketSnapshot};
use std::net::SocketAddr;
use tokio;

#[tokio::main]
async fn main() {
    println!("🌐 TLBSS Demo Dashboard");
    println!("Starting server on http://localhost:3000");

    let app = Router::new()
        .route("/", get(root))
        .route("/demo/:scenario", get(run_demo_api))
        .route("/health", get(health_check));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    include_str!("../../dashboard.html")
}

async fn health_check() -> &'static str {
    "OK"
}

async fn run_demo_api(
    axum::extract::Path(scenario): axum::extract::Path<String>,
) -> Json<DemoResult> {
    let snapshot = match scenario.as_str() {
        "normal" => MarketSnapshot::normal(),
        "reserve" => MarketSnapshot::reserve_shortage(),
        "capacity" => MarketSnapshot::capacity_shortage(),
        "network" => MarketSnapshot::network_overload(),
        "collapse" => MarketSnapshot::collapse_case(),
        _ => MarketSnapshot::normal(),
    };

    let result = run_full_demo(snapshot);
    Json(result)
}
<parameter name="filePath">/workspaces/M.V.R.ESPRINT1/src/bin/dashboard.rs