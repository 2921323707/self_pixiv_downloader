use axum::Json;

use crate::api::dto::HealthResponse;
use crate::api::error::ApiEnvelope;

pub(crate) async fn get_health() -> Json<ApiEnvelope<HealthResponse>> {
    Json(ApiEnvelope {
        data: HealthResponse {
            status: "ok".to_owned(),
        },
    })
}
