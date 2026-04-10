use crate::composition::{
    capabilities::{ACCESS_ORDER_MANAGEMENT_GATEWAY, IDENTITY_ACCESS_TOKEN_VERIFIER},
    context_registration::ApiContextRegistration,
    contribution::{
        ApiBackgroundJob, ApiContextContribution, ApiNamedBackgroundJob, ApiNamedReadinessCheck,
    },
    platform::ApiPlatform,
};
use crate::routes::fulfillment::{self, FulfillmentApiDoc};
use anyhow::Context;
use ordering_food_access_published::OrderManagementAccessGateway;
use ordering_food_bootstrap_core::{BootstrapRegistration, ContextDescriptor};
use ordering_food_fulfillment_integration::{
    AccessWorkflowActionAuthorizer, OrderingEventProjector, build_fulfillment_context_runtime,
};
use ordering_food_identity_published::AccessTokenVerifier;
use std::sync::Arc;
use tokio::{
    sync::oneshot,
    task::JoinHandle,
    time::{Duration, sleep},
};
use tracing::error;
use utoipa::OpenApi;

pub fn register_fulfillment() -> ApiContextRegistration {
    let descriptor = ContextDescriptor {
        id: "fulfillment",
        depends_on: &["identity", "access"],
    };

    ApiContextRegistration::without_migration(descriptor, fulfillment_bootstrap_registration)
}

fn fulfillment_bootstrap_registration(
    descriptor: ContextDescriptor,
) -> BootstrapRegistration<ApiPlatform, ApiContextContribution> {
    BootstrapRegistration::new(descriptor, move |platform: &ApiPlatform| {
        let context_id = descriptor.id;
        let pg_pool = platform.pg_pool.clone();
        let clock = platform.clock.clone();
        let access_gateway = platform
            .capabilities
            .resolve::<Arc<dyn OrderManagementAccessGateway>>(ACCESS_ORDER_MANAGEMENT_GATEWAY);
        let token_verifier = platform
            .capabilities
            .resolve::<Arc<dyn AccessTokenVerifier>>(IDENTITY_ACCESS_TOKEN_VERIFIER);
        async move {
            let access_gateway = access_gateway.ok_or_else(|| {
                std::io::Error::other(
                    "access capability `access.order_management_gateway` is not available",
                )
            })?;
            let token_verifier = token_verifier.ok_or_else(|| {
                std::io::Error::other(
                    "identity capability `identity.access_token_verifier` is not available",
                )
            })?;
            let workflow_action_authorizer =
                Arc::new(AccessWorkflowActionAuthorizer::new(access_gateway.clone()));
            let runtime =
                build_fulfillment_context_runtime(pg_pool, clock, workflow_action_authorizer);
            let module = runtime.module().clone();
            let projector = runtime.ordering_event_projector().clone();

            let mut contribution = ApiContextContribution::empty(context_id);
            contribution.add_readiness_check(ApiNamedReadinessCheck::always_ok(
                context_id,
                "module_ready",
            ));
            contribution.add_background_job(ApiNamedBackgroundJob::new(
                context_id,
                "ordering_event_projector",
                Box::new(OrderingEventProjectorBackgroundJob::new(projector)),
            ));
            contribution.add_route_mount(
                fulfillment::ORDER_ROUTE_PREFIX,
                crate::routes::fulfillment::router(module.clone())
                    .layer(axum::Extension(token_verifier)),
            );
            contribution.add_openapi_document(FulfillmentApiDoc::openapi());
            contribution.retain_private(module);

            Ok::<_, std::io::Error>(contribution)
        }
    })
}

struct OrderingEventProjectorBackgroundJob {
    projector: OrderingEventProjector,
    stop_tx: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl OrderingEventProjectorBackgroundJob {
    fn new(projector: OrderingEventProjector) -> Self {
        Self {
            projector,
            stop_tx: None,
            handle: None,
        }
    }
}

#[async_trait::async_trait]
impl ApiBackgroundJob for OrderingEventProjectorBackgroundJob {
    async fn start(&mut self) -> anyhow::Result<()> {
        if self.handle.is_some() {
            return Ok(());
        }

        let projector = self.projector.clone();
        let (stop_tx, mut stop_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    result = projector.project_once() => {
                        match result {
                            Ok(run_result) if run_result.scanned_count == 0 => {
                                sleep(Duration::from_millis(250)).await;
                            }
                            Ok(_) => {}
                            Err(projector_error) => {
                                error!(error = %projector_error, "fulfillment ordering projector iteration failed");
                                sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }
        });

        self.stop_tx = Some(stop_tx);
        self.handle = Some(handle);
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.handle.take() {
            handle
                .await
                .context("failed to join fulfillment ordering projector task")?;
        }

        Ok(())
    }
}
