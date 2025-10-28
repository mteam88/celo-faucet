use crate::faucet::FaucetService;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize)]
struct FaucetRequest {
    address: String,
}

#[derive(Serialize)]
struct FaucetResponse {
    #[serde(rename = "txHash")]
    tx_hash: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[handler]
async fn faucet_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let faucet_service = depot.get::<Arc<FaucetService>>("faucet_service").unwrap();

    let body = match req.parse_json::<FaucetRequest>().await {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to parse request body: {}", e);
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(ErrorResponse {
                error: "Invalid request body".to_string(),
            }));
            return;
        }
    };

    match faucet_service.send_native(&body.address).await {
        Ok(tx_hash) => {
            res.status_code(StatusCode::OK);
            res.render(Json(FaucetResponse { tx_hash }));
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("already_sent") {
                res.status_code(StatusCode::CONFLICT);
                res.render(Json(ErrorResponse {
                    error: "already_sent".to_string(),
                }));
            } else if error_msg.contains("Invalid") || error_msg.contains("address") {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(ErrorResponse {
                    error: format!("Invalid address: {}", error_msg),
                }));
            } else {
                error!("Faucet error: {}", e);
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(ErrorResponse {
                    error: "Failed to send transaction".to_string(),
                }));
            }
        }
    }
}

#[handler]
async fn healthz_handler(_req: &mut Request, res: &mut Response) {
    res.render(Json(HealthResponse {
        status: "ok".to_string(),
    }));
}

struct ServiceInjector {
    service: Arc<FaucetService>,
}

#[async_trait::async_trait]
impl Handler for ServiceInjector {
    async fn handle(
        &self,
        _req: &mut Request,
        depot: &mut Depot,
        _res: &mut Response,
        _ctrl: &mut FlowCtrl,
    ) {
        depot.insert("faucet_service", self.service.clone());
        // Continue to the next handlers in the chain
        _ctrl.call_next(_req, depot, _res).await;
    }
}

pub fn create_router(faucet_service: Arc<FaucetService>) -> Router {
    Router::new()
        .hoop(ServiceInjector {
            service: faucet_service,
        })
        // Serve only index.html at root
        .push(Router::with_path("/").get(StaticFile::new("web/index.html")))
        // Serve assets under /assets/* from web/assets
        .push(Router::with_path("/assets/<**path>").get(StaticDir::new(["web/assets"]).auto_list(false)))
        // temporarily disabled
        // .push(Router::with_path("/faucet").post(faucet_handler))
        .push(Router::with_path("/healthz").get(healthz_handler))
        // Serve only assets under /dist/* from web/dist
        .push(
            Router::with_path("/dist/<**path>").get(StaticDir::new(["web/dist"]).auto_list(false)),
        )
}
