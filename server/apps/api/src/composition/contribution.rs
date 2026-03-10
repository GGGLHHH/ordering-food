use crate::{app::AppState, error::AppError};
use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use axum::Router;
use std::{any::Any, collections::BTreeSet, sync::Arc};
use utoipa::openapi::OpenApi;

pub struct ApiRouteMount {
    pub path: &'static str,
    pub router: Router<AppState>,
}

#[async_trait]
pub trait ApiReadinessCheck: Send + Sync {
    async fn check(&self) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct ApiNamedReadinessCheck {
    pub context_id: &'static str,
    pub label: &'static str,
    probe: Arc<dyn ApiReadinessCheck>,
}

impl ApiNamedReadinessCheck {
    pub fn new(
        context_id: &'static str,
        label: &'static str,
        probe: Arc<dyn ApiReadinessCheck>,
    ) -> Self {
        Self {
            context_id,
            label,
            probe,
        }
    }

    pub async fn run(&self) -> Result<(), AppError> {
        self.probe.check().await
    }

    pub fn always_ok(context_id: &'static str, label: &'static str) -> Self {
        Self::new(context_id, label, Arc::new(AlwaysOkReadinessCheck))
    }
}

struct AlwaysOkReadinessCheck;

#[async_trait]
impl ApiReadinessCheck for AlwaysOkReadinessCheck {
    async fn check(&self) -> Result<(), AppError> {
        Ok(())
    }
}

#[async_trait]
pub trait ApiLifecycleHook: Send + Sync {
    async fn run(&self) -> Result<()>;
}

pub struct ApiNamedLifecycleHook {
    pub context_id: &'static str,
    pub label: &'static str,
    hook: Arc<dyn ApiLifecycleHook>,
}

impl ApiNamedLifecycleHook {
    pub fn new(
        context_id: &'static str,
        label: &'static str,
        hook: Arc<dyn ApiLifecycleHook>,
    ) -> Self {
        Self {
            context_id,
            label,
            hook,
        }
    }

    pub async fn run(&self, phase: &'static str) -> Result<()> {
        self.hook.run().await.with_context(|| {
            format!(
                "{phase} hook `{}` failed for context `{}`",
                self.label, self.context_id
            )
        })
    }
}

#[async_trait]
pub trait ApiBackgroundJob: Send {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
}

pub struct ApiNamedBackgroundJob {
    pub context_id: &'static str,
    pub label: &'static str,
    job: Box<dyn ApiBackgroundJob>,
}

impl ApiNamedBackgroundJob {
    pub fn new(
        context_id: &'static str,
        label: &'static str,
        job: Box<dyn ApiBackgroundJob>,
    ) -> Self {
        Self {
            context_id,
            label,
            job,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.job.start().await.with_context(|| {
            format!(
                "background job `{}` failed to start for context `{}`",
                self.label, self.context_id
            )
        })
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.job.stop().await.with_context(|| {
            format!(
                "background job `{}` failed to stop for context `{}`",
                self.label, self.context_id
            )
        })
    }
}

pub struct ApiContextContribution {
    context_id: &'static str,
    route_mounts: Vec<ApiRouteMount>,
    openapi_documents: Vec<OpenApi>,
    readiness_checks: Vec<ApiNamedReadinessCheck>,
    startup_hooks: Vec<ApiNamedLifecycleHook>,
    shutdown_hooks: Vec<ApiNamedLifecycleHook>,
    background_jobs: Vec<ApiNamedBackgroundJob>,
    private_runtime_objects: Vec<Box<dyn Any + Send + Sync>>,
}

impl ApiContextContribution {
    pub fn empty(context_id: &'static str) -> Self {
        Self {
            context_id,
            route_mounts: Vec::new(),
            openapi_documents: Vec::new(),
            readiness_checks: Vec::new(),
            startup_hooks: Vec::new(),
            shutdown_hooks: Vec::new(),
            background_jobs: Vec::new(),
            private_runtime_objects: Vec::new(),
        }
    }

    pub fn add_route_mount(&mut self, path: &'static str, router: Router<AppState>) {
        self.route_mounts.push(ApiRouteMount { path, router });
    }

    pub fn add_openapi_document(&mut self, openapi: OpenApi) {
        self.openapi_documents.push(openapi);
    }

    pub fn add_readiness_check(&mut self, check: ApiNamedReadinessCheck) {
        self.readiness_checks.push(check);
    }

    pub fn add_startup_hook(&mut self, hook: ApiNamedLifecycleHook) {
        self.startup_hooks.push(hook);
    }

    pub fn add_shutdown_hook(&mut self, hook: ApiNamedLifecycleHook) {
        self.shutdown_hooks.push(hook);
    }

    pub fn add_background_job(&mut self, job: ApiNamedBackgroundJob) {
        self.background_jobs.push(job);
    }

    pub fn retain_private<T>(&mut self, value: T)
    where
        T: Any + Send + Sync + 'static,
    {
        self.private_runtime_objects.push(Box::new(value));
    }

    pub fn into_parts(self) -> ApiContextContributionParts {
        ApiContextContributionParts {
            context_id: self.context_id,
            route_mounts: self.route_mounts,
            openapi_documents: self.openapi_documents,
            readiness_checks: self.readiness_checks,
            startup_hooks: self.startup_hooks,
            shutdown_hooks: self.shutdown_hooks,
            background_jobs: self.background_jobs,
            private_runtime_objects: self.private_runtime_objects,
        }
    }
}

pub struct ApiContextContributionParts {
    pub context_id: &'static str,
    pub route_mounts: Vec<ApiRouteMount>,
    pub openapi_documents: Vec<OpenApi>,
    pub readiness_checks: Vec<ApiNamedReadinessCheck>,
    pub startup_hooks: Vec<ApiNamedLifecycleHook>,
    pub shutdown_hooks: Vec<ApiNamedLifecycleHook>,
    pub background_jobs: Vec<ApiNamedBackgroundJob>,
    pub private_runtime_objects: Vec<Box<dyn Any + Send + Sync>>,
}

pub struct ApiLifecycleRuntime {
    shutdown_hooks: Vec<ApiNamedLifecycleHook>,
    background_jobs: Vec<ApiNamedBackgroundJob>,
}

impl ApiLifecycleRuntime {
    pub fn new(
        shutdown_hooks: Vec<ApiNamedLifecycleHook>,
        background_jobs: Vec<ApiNamedBackgroundJob>,
    ) -> Self {
        Self {
            shutdown_hooks,
            background_jobs,
        }
    }

    pub async fn start(
        startup_hooks: &[ApiNamedLifecycleHook],
        shutdown_hooks: &[ApiNamedLifecycleHook],
        background_jobs: &mut [ApiNamedBackgroundJob],
    ) -> Result<()> {
        let mut started_context_ids = BTreeSet::new();

        for hook in startup_hooks {
            if let Err(error) = hook.run("startup").await {
                let shutdown_errors = run_shutdown_hooks_for_contexts(
                    shutdown_hooks,
                    &started_context_ids,
                    "startup_rollback",
                )
                .await;
                return Err(combine_errors("startup", error, shutdown_errors));
            }

            started_context_ids.insert(hook.context_id);
        }

        for (started_jobs, job) in background_jobs.iter_mut().enumerate() {
            if let Err(error) = job.start().await {
                let stop_errors = stop_started_jobs(&mut background_jobs[..started_jobs]).await;
                let shutdown_errors = run_shutdown_hooks_for_contexts(
                    shutdown_hooks,
                    &started_context_ids,
                    "startup_rollback",
                )
                .await;

                return Err(combine_errors(
                    "startup",
                    error,
                    merge_error_vectors(stop_errors, shutdown_errors),
                ));
            }
        }

        Ok(())
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let job_errors = stop_started_jobs(&mut self.background_jobs).await;
        let hook_errors = run_shutdown_hooks(&self.shutdown_hooks, "shutdown").await;
        let errors = merge_error_vectors(job_errors, hook_errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(aggregate_errors("shutdown", errors))
        }
    }
}

pub async fn stop_started_jobs(
    background_jobs: &mut [ApiNamedBackgroundJob],
) -> Vec<anyhow::Error> {
    let mut errors = Vec::new();

    for job in background_jobs.iter_mut().rev() {
        if let Err(error) = job.stop().await {
            errors.push(error);
        }
    }

    errors
}

async fn run_shutdown_hooks_for_contexts(
    shutdown_hooks: &[ApiNamedLifecycleHook],
    context_ids: &BTreeSet<&'static str>,
    phase: &'static str,
) -> Vec<anyhow::Error> {
    let mut errors = Vec::new();

    for hook in shutdown_hooks.iter().rev() {
        if context_ids.contains(&hook.context_id)
            && let Err(error) = hook.run(phase).await
        {
            errors.push(error);
        }
    }

    errors
}

async fn run_shutdown_hooks(
    shutdown_hooks: &[ApiNamedLifecycleHook],
    phase: &'static str,
) -> Vec<anyhow::Error> {
    let mut errors = Vec::new();

    for hook in shutdown_hooks.iter().rev() {
        if let Err(error) = hook.run(phase).await {
            errors.push(error);
        }
    }

    errors
}

fn merge_error_vectors(
    mut left: Vec<anyhow::Error>,
    right: Vec<anyhow::Error>,
) -> Vec<anyhow::Error> {
    left.extend(right);
    left
}

fn combine_errors(
    phase: &'static str,
    primary_error: anyhow::Error,
    secondary_errors: Vec<anyhow::Error>,
) -> anyhow::Error {
    if secondary_errors.is_empty() {
        primary_error.context(format!("{phase} phase failed"))
    } else {
        let mut messages = vec![primary_error.to_string()];
        messages.extend(secondary_errors.into_iter().map(|error| error.to_string()));
        anyhow!(
            "{phase} phase failed with {} error(s): {}",
            messages.len(),
            messages.join(" | ")
        )
    }
}

fn aggregate_errors(phase: &'static str, errors: Vec<anyhow::Error>) -> anyhow::Error {
    let messages = errors
        .into_iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    anyhow!(
        "{phase} phase encountered {} error(s): {}",
        messages.len(),
        messages.join(" | ")
    )
}
