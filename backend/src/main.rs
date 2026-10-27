use medpulse_api::{ClinicalPool, IdentityPool, router};
use thiserror::Error;
use tokio::net::TcpListener;

#[derive(Debug, Error)]
enum StartupError {
    #[error("required database configuration is missing")]
    MissingDatabaseConfiguration,
    #[error("identity database connection failed")]
    IdentityConnection,
    #[error("clinical database connection failed")]
    ClinicalConnection,
    #[error("could not bind the API listener")]
    Listener,
    #[error("API server failed")]
    Server,
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    let identity_url = std::env::var("IDENTITY_DATABASE_URL")
        .map_err(|_| StartupError::MissingDatabaseConfiguration)?;
    let clinical_url = std::env::var("CLINICAL_DATABASE_URL")
        .map_err(|_| StartupError::MissingDatabaseConfiguration)?;

    let identity = IdentityPool::connect(&identity_url)
        .await
        .map_err(|_| StartupError::IdentityConnection)?;
    let clinical = ClinicalPool::connect(&clinical_url)
        .await
        .map_err(|_| StartupError::ClinicalConnection)?;
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .map_err(|_| StartupError::Listener)?;

    let result = axum::serve(listener, router()).await.map_err(|_| StartupError::Server);
    identity.close().await;
    clinical.close().await;
    result
}
