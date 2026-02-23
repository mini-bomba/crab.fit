use std::{env, future::Future, sync::Arc};

use axum::{extract, http::HeaderMap};
use chrono::{Duration, Utc};
use common::Adaptor;
use futures::{future::Shared};
use tokio::{sync::Mutex, time::sleep};
use tracing::{error, info};

use crate::{ApiState, State, errors::ApiError};

#[utoipa::path(
    get,
    path = "/tasks/cleanup",
    responses(
        (status = 200, description = "Cleanup complete"),
        (status = 401, description = "Missing or incorrect X-Cron-Key header"),
        (status = 429, description = "Too many requests"),
    ),
    security((), ("cron-key" = [])),
    tag = "tasks",
)]
/// Delete events older than 3 months
pub async fn cleanup<A: Adaptor>(
    extract::State(state): State<A>,
    headers: HeaderMap,
) -> Result<(), ApiError<A>> {
    // Check cron key
    let cron_key_header: String = headers
        .get("X-Cron-Key")
        .map(|k| k.to_str().unwrap_or_default().into())
        .unwrap_or_default();
    let env_key = env::var("CRON_KEY").unwrap_or_default();
    if !env_key.is_empty() && cron_key_header != env_key {
        return Err(ApiError::NotAuthorized);
    }

    do_cleanup(&state.lock().await.adaptor)
        .await
        .map_err(ApiError::AdaptorError)
}

async fn do_cleanup<A: Adaptor>(adaptor: &A) -> Result<(), A::Error> {
    info!("Running cleanup task");

    let result = adaptor
        .delete_events(Utc::now() - Duration::days(30))
        .await?;

    info!(
        "Cleanup successful: {} events and {} people removed",
        result.event_count, result.person_count
    );
    Ok(())
}

pub async fn cleanup_worker<A: Adaptor>(
    exit_signal: Shared<impl Future<Output = ()>>,
    shared_state: Arc<Mutex<ApiState<A>>>,
) {
    loop {
        tokio::select! {
            biased;
            _ = exit_signal.clone() => return,
            _ = sleep(std::time::Duration::from_hours(1)) => (),
        };
        
        if let Err(e) = do_cleanup(&shared_state.lock().await.adaptor).await {
            error!("Hourly cleanup failed: {e:?}");
        }
    }
}
