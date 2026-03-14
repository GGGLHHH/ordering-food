use crate::{
    composition::{
        context_registration::ApiContextRegistration,
        contexts,
        contribution::{
            ApiContextContribution, ApiLifecycleRuntime, ApiNamedBackgroundJob,
            ApiNamedLifecycleHook, ApiNamedReadinessCheck, ApiRouteMount,
        },
        platform::ApiPlatform,
    },
    readiness::{CompositeReadiness, ReadinessProbe},
};
use anyhow::{Result, anyhow};
use ordering_food_bootstrap_core::{BootstrapRegistry, MigrationRegistry, RegistryError};
use std::{any::Any, sync::Arc};
use utoipa::openapi::OpenApi;

pub struct ApiCompositionRuntime {
    readiness: Arc<dyn ReadinessProbe>,
    route_mounts: Vec<ApiRouteMount>,
    openapi_documents: Vec<OpenApi>,
    lifecycle: ApiLifecycleRuntime,
    private_runtime_objects: Vec<Box<dyn Any + Send + Sync>>,
}

pub struct ApiRunParts {
    pub readiness: Arc<dyn ReadinessProbe>,
    pub route_mounts: Vec<ApiRouteMount>,
    pub openapi_documents: Vec<OpenApi>,
    pub lifecycle: ApiLifecycleRuntime,
    pub keepalive: Vec<Box<dyn Any + Send + Sync>>,
}

impl ApiCompositionRuntime {
    pub fn into_run_parts(self) -> ApiRunParts {
        ApiRunParts {
            readiness: self.readiness,
            route_mounts: self.route_mounts,
            openapi_documents: self.openapi_documents,
            lifecycle: self.lifecycle,
            keepalive: self.private_runtime_objects,
        }
    }
}

pub async fn prepare_runtime(platform: ApiPlatform) -> Result<ApiCompositionRuntime> {
    prepare_runtime_with_registrations(platform, contexts::registrations()).await
}

async fn prepare_runtime_with_registrations(
    platform: ApiPlatform,
    registrations: Vec<ApiContextRegistration>,
) -> Result<ApiCompositionRuntime> {
    build_migration_registry(&registrations)?
        .run_all(&platform)
        .await
        .map_err(map_registry_error)?;

    let contributions = build_bootstrap_registry(&registrations)?
        .bootstrap_all(&platform)
        .await
        .map_err(map_registry_error)?;

    assemble_runtime(platform, contributions).await
}

fn build_migration_registry(
    registrations: &[ApiContextRegistration],
) -> Result<MigrationRegistry<ApiPlatform>, RegistryError> {
    MigrationRegistry::new(
        registrations
            .iter()
            .filter_map(ApiContextRegistration::migration_registration)
            .collect(),
    )
}

fn build_bootstrap_registry(
    registrations: &[ApiContextRegistration],
) -> Result<BootstrapRegistry<ApiPlatform, ApiContextContribution>, RegistryError> {
    BootstrapRegistry::new(
        registrations
            .iter()
            .map(ApiContextRegistration::bootstrap_registration)
            .collect(),
    )
}

async fn assemble_runtime(
    platform: ApiPlatform,
    contributions: Vec<ApiContextContribution>,
) -> Result<ApiCompositionRuntime> {
    let mut route_mounts = Vec::new();
    let mut openapi_documents = Vec::new();
    let mut readiness_checks = Vec::<ApiNamedReadinessCheck>::new();
    let mut startup_hooks = Vec::<ApiNamedLifecycleHook>::new();
    let mut shutdown_hooks = Vec::<ApiNamedLifecycleHook>::new();
    let mut background_jobs = Vec::<ApiNamedBackgroundJob>::new();
    let mut private_runtime_objects = Vec::<Box<dyn Any + Send + Sync>>::new();

    for contribution in contributions {
        let parts = contribution.into_parts();
        route_mounts.extend(parts.route_mounts);
        openapi_documents.extend(parts.openapi_documents);
        readiness_checks.extend(parts.readiness_checks);
        startup_hooks.extend(parts.startup_hooks);
        shutdown_hooks.extend(parts.shutdown_hooks);
        background_jobs.extend(parts.background_jobs);
        private_runtime_objects.extend(parts.private_runtime_objects);
        let _ = parts.context_id;
    }

    ApiLifecycleRuntime::start(&startup_hooks, &shutdown_hooks, &mut background_jobs).await?;
    let readiness = Arc::new(CompositeReadiness::new(
        platform.pg_pool,
        platform.redis_client,
        readiness_checks,
    ));

    Ok(ApiCompositionRuntime {
        readiness,
        route_mounts,
        openapi_documents,
        lifecycle: ApiLifecycleRuntime::new(shutdown_hooks, background_jobs),
        private_runtime_objects,
    })
}

fn map_registry_error(error: RegistryError) -> anyhow::Error {
    anyhow!(error).context("failed to prepare application context registries")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{
        context_registration::ApiContextRegistration,
        contribution::{
            ApiBackgroundJob, ApiContextContribution, ApiLifecycleHook, ApiNamedBackgroundJob,
            ApiNamedLifecycleHook,
        },
        platform::ApiPlatform,
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use ordering_food_bootstrap_core::ContextDescriptor;
    use std::sync::{Arc, Mutex};

    fn test_platform() -> ApiPlatform {
        ApiPlatform::new(
            crate::config::Settings::from_overrides(std::iter::empty::<(String, String)>())
                .unwrap(),
            sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://ordering_food:ordering_food@127.0.0.1:5432/ordering_food")
                .unwrap(),
            redis::Client::open("redis://127.0.0.1:6379").unwrap(),
        )
    }

    struct RecordingHook {
        events: Arc<Mutex<Vec<String>>>,
        event: String,
    }

    #[async_trait]
    impl ApiLifecycleHook for RecordingHook {
        async fn run(&self) -> Result<()> {
            self.events.lock().unwrap().push(self.event.clone());
            Ok(())
        }
    }

    struct RecordingJob {
        events: Arc<Mutex<Vec<String>>>,
        label: String,
        fail_on_stop: bool,
    }

    #[async_trait]
    impl ApiBackgroundJob for RecordingJob {
        async fn start(&mut self) -> Result<()> {
            self.events
                .lock()
                .unwrap()
                .push(format!("start:{}", self.label));
            Ok(())
        }

        async fn stop(&mut self) -> Result<()> {
            self.events
                .lock()
                .unwrap()
                .push(format!("stop:{}", self.label));
            if self.fail_on_stop {
                return Err(anyhow!("stop failed for {}", self.label));
            }
            Ok(())
        }
    }

    fn fake_registration(descriptor: ContextDescriptor) -> ApiContextRegistration {
        fn migration_registration(
            descriptor: ContextDescriptor,
        ) -> ordering_food_bootstrap_core::MigrationRegistration<ApiPlatform> {
            ordering_food_bootstrap_core::MigrationRegistration::new(
                descriptor,
                move |_platform: &ApiPlatform| async move { Ok::<_, std::io::Error>(()) },
            )
        }

        fn bootstrap_registration(
            descriptor: ContextDescriptor,
        ) -> ordering_food_bootstrap_core::BootstrapRegistration<ApiPlatform, ApiContextContribution>
        {
            ordering_food_bootstrap_core::BootstrapRegistration::new(
                descriptor,
                move |_platform: &ApiPlatform| {
                    let context_id = descriptor.id;
                    async move { Ok::<_, std::io::Error>(ApiContextContribution::empty(context_id)) }
                },
            )
        }

        ApiContextRegistration::new(descriptor, migration_registration, bootstrap_registration)
    }

    #[tokio::test]
    async fn registry_sorts_descriptors_topologically() {
        let registrations = vec![
            fake_registration(ContextDescriptor {
                id: "ordering",
                depends_on: &["identity"],
            }),
            fake_registration(ContextDescriptor {
                id: "identity",
                depends_on: &[],
            }),
        ];

        let migration_registry = build_migration_registry(&registrations).unwrap();
        assert_eq!(
            migration_registry
                .descriptors()
                .into_iter()
                .map(|descriptor| descriptor.id)
                .collect::<Vec<_>>(),
            vec!["identity", "ordering"]
        );
    }

    #[tokio::test]
    async fn migration_registry_skips_contexts_without_migration_runner() {
        fn bootstrap_registration(
            descriptor: ContextDescriptor,
        ) -> ordering_food_bootstrap_core::BootstrapRegistration<ApiPlatform, ApiContextContribution>
        {
            ordering_food_bootstrap_core::BootstrapRegistration::new(
                descriptor,
                move |_platform: &ApiPlatform| {
                    let context_id = descriptor.id;
                    async move { Ok::<_, std::io::Error>(ApiContextContribution::empty(context_id)) }
                },
            )
        }

        let registrations = vec![
            ApiContextRegistration::without_migration(
                ContextDescriptor {
                    id: "menu",
                    depends_on: &[],
                },
                bootstrap_registration,
            ),
            fake_registration(ContextDescriptor {
                id: "identity",
                depends_on: &[],
            }),
        ];

        let migration_registry = build_migration_registry(&registrations).unwrap();
        assert_eq!(
            migration_registry
                .descriptors()
                .into_iter()
                .map(|descriptor| descriptor.id)
                .collect::<Vec<_>>(),
            vec!["identity"]
        );
    }

    #[tokio::test]
    async fn lifecycle_runs_startup_and_shutdown_in_expected_order() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut identity = ApiContextContribution::empty("identity");
        identity.add_startup_hook(ApiNamedLifecycleHook::new(
            "identity",
            "startup",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "startup:identity".to_string(),
            }),
        ));
        identity.add_shutdown_hook(ApiNamedLifecycleHook::new(
            "identity",
            "shutdown",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "shutdown:identity".to_string(),
            }),
        ));

        let mut ordering = ApiContextContribution::empty("ordering");
        ordering.add_startup_hook(ApiNamedLifecycleHook::new(
            "ordering",
            "startup",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "startup:ordering".to_string(),
            }),
        ));
        ordering.add_shutdown_hook(ApiNamedLifecycleHook::new(
            "ordering",
            "shutdown",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "shutdown:ordering".to_string(),
            }),
        ));
        ordering.add_background_job(ApiNamedBackgroundJob::new(
            "ordering",
            "job",
            Box::new(RecordingJob {
                events: Arc::clone(&events),
                label: "ordering".to_string(),
                fail_on_stop: false,
            }),
        ));

        let runtime = assemble_runtime(test_platform(), vec![identity, ordering])
            .await
            .unwrap();
        runtime.lifecycle.shutdown().await.unwrap();

        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                "startup:identity",
                "startup:ordering",
                "start:ordering",
                "stop:ordering",
                "shutdown:ordering",
                "shutdown:identity",
            ]
        );
    }

    #[tokio::test]
    async fn startup_failure_rolls_back_previously_started_contexts() {
        struct FailingHook;

        #[async_trait]
        impl ApiLifecycleHook for FailingHook {
            async fn run(&self) -> Result<()> {
                Err(anyhow!("boom"))
            }
        }

        let events = Arc::new(Mutex::new(Vec::new()));
        let mut identity = ApiContextContribution::empty("identity");
        identity.add_startup_hook(ApiNamedLifecycleHook::new(
            "identity",
            "startup",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "startup:identity".to_string(),
            }),
        ));
        identity.add_shutdown_hook(ApiNamedLifecycleHook::new(
            "identity",
            "shutdown",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "shutdown:identity".to_string(),
            }),
        ));

        let mut ordering = ApiContextContribution::empty("ordering");
        ordering.add_startup_hook(ApiNamedLifecycleHook::new(
            "ordering",
            "startup",
            Arc::new(FailingHook),
        ));
        ordering.add_shutdown_hook(ApiNamedLifecycleHook::new(
            "ordering",
            "shutdown",
            Arc::new(RecordingHook {
                events: Arc::clone(&events),
                event: "shutdown:ordering".to_string(),
            }),
        ));

        let error = match assemble_runtime(test_platform(), vec![identity, ordering]).await {
            Ok(_) => panic!("expected startup failure"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("startup phase failed"));
        assert_eq!(
            events.lock().unwrap().as_slice(),
            ["startup:identity", "shutdown:identity"]
        );
    }

    #[tokio::test]
    async fn migration_failure_prevents_bootstrap() {
        struct FailingRegistration;

        impl FailingRegistration {
            fn migration_registration(
                descriptor: ContextDescriptor,
            ) -> ordering_food_bootstrap_core::MigrationRegistration<ApiPlatform> {
                ordering_food_bootstrap_core::MigrationRegistration::new(
                    descriptor,
                    |_platform: &ApiPlatform| async move { Err::<(), _>(std::io::Error::other("boom")) },
                )
            }

            fn bootstrap_registration(
                descriptor: ContextDescriptor,
            ) -> ordering_food_bootstrap_core::BootstrapRegistration<
                ApiPlatform,
                ApiContextContribution,
            > {
                ordering_food_bootstrap_core::BootstrapRegistration::new(
                    descriptor,
                    move |_platform: &ApiPlatform| {
                        let context_id = descriptor.id;
                        async move { Ok::<_, std::io::Error>(ApiContextContribution::empty(context_id)) }
                    },
                )
            }
        }

        let error = match prepare_runtime_with_registrations(
            test_platform(),
            vec![ApiContextRegistration::new(
                ContextDescriptor {
                    id: "identity",
                    depends_on: &[],
                },
                FailingRegistration::migration_registration,
                FailingRegistration::bootstrap_registration,
            )],
        )
        .await
        {
            Ok(_) => panic!("expected runtime preparation failure"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("failed to prepare application context registries")
        );
    }

    #[tokio::test]
    async fn bootstrap_failure_surfaces_as_runtime_error() {
        struct FailingRegistration;

        impl FailingRegistration {
            fn migration_registration(
                descriptor: ContextDescriptor,
            ) -> ordering_food_bootstrap_core::MigrationRegistration<ApiPlatform> {
                ordering_food_bootstrap_core::MigrationRegistration::new(
                    descriptor,
                    |_platform: &ApiPlatform| async move { Ok::<_, std::io::Error>(()) },
                )
            }

            fn bootstrap_registration(
                descriptor: ContextDescriptor,
            ) -> ordering_food_bootstrap_core::BootstrapRegistration<
                ApiPlatform,
                ApiContextContribution,
            > {
                ordering_food_bootstrap_core::BootstrapRegistration::new(
                    descriptor,
                    |_platform: &ApiPlatform| async move {
                        Err::<ApiContextContribution, _>(std::io::Error::other("boom"))
                    },
                )
            }
        }

        let error = match prepare_runtime_with_registrations(
            test_platform(),
            vec![ApiContextRegistration::new(
                ContextDescriptor {
                    id: "identity",
                    depends_on: &[],
                },
                FailingRegistration::migration_registration,
                FailingRegistration::bootstrap_registration,
            )],
        )
        .await
        {
            Ok(_) => panic!("expected runtime preparation failure"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("failed to prepare application context registries")
        );
    }

    #[tokio::test]
    async fn shutdown_collects_all_errors_in_reverse_order() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let lifecycle = ApiLifecycleRuntime::new(
            vec![
                ApiNamedLifecycleHook::new(
                    "identity",
                    "shutdown-identity",
                    Arc::new(RecordingHook {
                        events: Arc::clone(&events),
                        event: "shutdown:identity".to_string(),
                    }),
                ),
                ApiNamedLifecycleHook::new(
                    "ordering",
                    "shutdown-ordering",
                    Arc::new(RecordingHook {
                        events: Arc::clone(&events),
                        event: "shutdown:ordering".to_string(),
                    }),
                ),
            ],
            vec![
                ApiNamedBackgroundJob::new(
                    "identity",
                    "job-identity",
                    Box::new(RecordingJob {
                        events: Arc::clone(&events),
                        label: "identity".to_string(),
                        fail_on_stop: true,
                    }),
                ),
                ApiNamedBackgroundJob::new(
                    "ordering",
                    "job-ordering",
                    Box::new(RecordingJob {
                        events: Arc::clone(&events),
                        label: "ordering".to_string(),
                        fail_on_stop: true,
                    }),
                ),
            ],
        );

        let error = lifecycle.shutdown().await.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("shutdown phase encountered 2 error")
        );
        assert_eq!(
            events.lock().unwrap().as_slice(),
            [
                "stop:ordering",
                "stop:identity",
                "shutdown:ordering",
                "shutdown:identity",
            ]
        );
    }
}
