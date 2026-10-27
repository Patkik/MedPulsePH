//! Early infrastructure only. No authentication, NFC, or clinical routes exist yet.

use axum::{Router, http::StatusCode, routing::get};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// A pool for identity data. Clinical repositories must never accept this type.
pub struct IdentityPool(PgPool);

/// A pool for clinical data. Clinical operations require a verified session.
pub struct ClinicalPool(PgPool);

impl IdentityPool {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        PgPoolOptions::new().max_connections(5).connect(database_url).await.map(Self)
    }

    pub async fn close(&self) {
        self.0.close().await;
    }
}

impl ClinicalPool {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        PgPoolOptions::new().max_connections(5).connect(database_url).await.map(Self)
    }

    pub async fn close(&self) {
        self.0.close().await;
    }
}

/// Only process liveness is exposed. It does not assert database readiness.
pub fn router() -> Router {
    Router::new().route("/health/live", get(|| async { StatusCode::NO_CONTENT }))
}

#[cfg(test)]
mod tests {
    use super::router;
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;

    #[tokio::test]
    async fn only_liveness_is_exposed() -> Result<(), Box<dyn std::error::Error>> {
        let live = Request::builder().uri("/health/live").body(Body::empty())?;
        assert_eq!(router().oneshot(live).await?.status(), StatusCode::NO_CONTENT);

        for path in ["/", "/api/v1/fhir/Patient", "/api/v1/auth/login", "/v1/tap"] {
            let request = Request::builder().uri(path).body(Body::empty())?;
            assert_eq!(router().oneshot(request).await?.status(), StatusCode::NOT_FOUND);
        }
        Ok(())
    }
}
