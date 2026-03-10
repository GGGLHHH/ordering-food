use crate::{BoxError, ContextDescriptor};
use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait MigrationRunner<P>: Send + Sync {
    async fn run(&self, platform: &P) -> Result<(), BoxError>;
}

#[async_trait]
impl<P, F, Fut, E> MigrationRunner<P> for F
where
    P: Sync,
    F: Send + Sync + 'static + Fn(&P) -> Fut,
    Fut: Future<Output = Result<(), E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    async fn run(&self, platform: &P) -> Result<(), BoxError> {
        (self)(platform)
            .await
            .map_err(|error| Box::new(error) as BoxError)
    }
}

pub struct MigrationRegistration<P> {
    pub descriptor: ContextDescriptor,
    runner: Box<dyn MigrationRunner<P>>,
}

impl<P> MigrationRegistration<P> {
    pub fn new<R>(descriptor: ContextDescriptor, runner: R) -> Self
    where
        R: MigrationRunner<P> + 'static,
    {
        Self {
            descriptor,
            runner: Box::new(runner),
        }
    }

    pub async fn run(&self, platform: &P) -> Result<(), BoxError> {
        self.runner.run(platform).await
    }
}

#[async_trait]
pub trait BootstrapRunner<P, C>: Send + Sync {
    async fn run(&self, platform: &P) -> Result<C, BoxError>;
}

#[async_trait]
impl<P, C, F, Fut, E> BootstrapRunner<P, C> for F
where
    P: Sync,
    C: Send,
    F: Send + Sync + 'static + Fn(&P) -> Fut,
    Fut: Future<Output = Result<C, E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    async fn run(&self, platform: &P) -> Result<C, BoxError> {
        (self)(platform)
            .await
            .map_err(|error| Box::new(error) as BoxError)
    }
}

pub struct BootstrapRegistration<P, C> {
    pub descriptor: ContextDescriptor,
    runner: Box<dyn BootstrapRunner<P, C>>,
}

impl<P, C> BootstrapRegistration<P, C> {
    pub fn new<R>(descriptor: ContextDescriptor, runner: R) -> Self
    where
        R: BootstrapRunner<P, C> + 'static,
    {
        Self {
            descriptor,
            runner: Box::new(runner),
        }
    }

    pub async fn run(&self, platform: &P) -> Result<C, BoxError> {
        self.runner.run(platform).await
    }
}
