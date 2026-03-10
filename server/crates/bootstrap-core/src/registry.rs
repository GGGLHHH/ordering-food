use crate::{
    BootstrapRegistration, ContextDescriptor, ContextOrderPlanner, MigrationRegistration,
    RegistryError,
};
use std::collections::BTreeMap;

pub struct MigrationRegistry<P> {
    registrations: Vec<MigrationRegistration<P>>,
}

impl<P> MigrationRegistry<P> {
    pub fn new(registrations: Vec<MigrationRegistration<P>>) -> Result<Self, RegistryError> {
        let ordered_ids = ContextOrderPlanner::plan(
            registrations
                .iter()
                .map(|registration| registration.descriptor),
        )?;
        let mut registrations_by_id = registrations
            .into_iter()
            .map(|registration| (registration.descriptor.id, registration))
            .collect::<BTreeMap<_, _>>();
        let registrations = ordered_ids
            .into_iter()
            .map(|descriptor| {
                registrations_by_id
                    .remove(descriptor.id)
                    .expect("migration registration missing after planning")
            })
            .collect();

        Ok(Self { registrations })
    }

    pub async fn run_all(&self, platform: &P) -> Result<(), RegistryError> {
        for registration in &self.registrations {
            registration.run(platform).await.map_err(|source| {
                RegistryError::phase_failed("migration", registration.descriptor.id, source)
            })?;
        }

        Ok(())
    }

    pub fn descriptors(&self) -> Vec<ContextDescriptor> {
        self.registrations
            .iter()
            .map(|registration| registration.descriptor)
            .collect()
    }
}

pub struct BootstrapRegistry<P, C> {
    registrations: Vec<BootstrapRegistration<P, C>>,
}

impl<P, C> BootstrapRegistry<P, C> {
    pub fn new(registrations: Vec<BootstrapRegistration<P, C>>) -> Result<Self, RegistryError> {
        let ordered_ids = ContextOrderPlanner::plan(
            registrations
                .iter()
                .map(|registration| registration.descriptor),
        )?;
        let mut registrations_by_id = registrations
            .into_iter()
            .map(|registration| (registration.descriptor.id, registration))
            .collect::<BTreeMap<_, _>>();
        let registrations = ordered_ids
            .into_iter()
            .map(|descriptor| {
                registrations_by_id
                    .remove(descriptor.id)
                    .expect("bootstrap registration missing after planning")
            })
            .collect();

        Ok(Self { registrations })
    }

    pub async fn bootstrap_all(&self, platform: &P) -> Result<Vec<C>, RegistryError> {
        let mut contributions = Vec::with_capacity(self.registrations.len());

        for registration in &self.registrations {
            contributions.push(registration.run(platform).await.map_err(|source| {
                RegistryError::phase_failed("bootstrap", registration.descriptor.id, source)
            })?);
        }

        Ok(contributions)
    }

    pub fn descriptors(&self) -> Vec<ContextDescriptor> {
        self.registrations
            .iter()
            .map(|registration| registration.descriptor)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn migration_registry_runs_in_topological_order() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let registry = MigrationRegistry::new(vec![
            MigrationRegistration::new(
                ContextDescriptor {
                    id: "ordering",
                    depends_on: &["identity"],
                },
                {
                    let events = Arc::clone(&events);
                    move |_platform: &()| {
                        let events = Arc::clone(&events);
                        async move {
                            events.lock().unwrap().push("ordering".to_string());
                            Ok::<_, std::io::Error>(())
                        }
                    }
                },
            ),
            MigrationRegistration::new(
                ContextDescriptor {
                    id: "identity",
                    depends_on: &[],
                },
                {
                    let events = Arc::clone(&events);
                    move |_platform: &()| {
                        let events = Arc::clone(&events);
                        async move {
                            events.lock().unwrap().push("identity".to_string());
                            Ok::<_, std::io::Error>(())
                        }
                    }
                },
            ),
        ])
        .unwrap();

        registry.run_all(&()).await.unwrap();

        assert_eq!(events.lock().unwrap().as_slice(), ["identity", "ordering"]);
    }

    #[tokio::test]
    async fn bootstrap_registry_returns_bootstrapped_values_in_order() {
        let registry = BootstrapRegistry::new(vec![
            BootstrapRegistration::new(
                ContextDescriptor {
                    id: "ordering",
                    depends_on: &["identity"],
                },
                |_platform: &()| async move { Ok::<_, std::io::Error>("ordering") },
            ),
            BootstrapRegistration::new(
                ContextDescriptor {
                    id: "identity",
                    depends_on: &[],
                },
                |_platform: &()| async move { Ok::<_, std::io::Error>("identity") },
            ),
        ])
        .unwrap();

        let values = registry.bootstrap_all(&()).await.unwrap();
        assert_eq!(values, vec!["identity", "ordering"]);
    }

    #[tokio::test]
    async fn migration_failure_stops_registry() {
        let registry = MigrationRegistry::new(vec![MigrationRegistration::new(
            ContextDescriptor {
                id: "identity",
                depends_on: &[],
            },
            |_platform: &()| async move { Err::<(), _>(std::io::Error::other("boom")) },
        )])
        .unwrap();

        let error = registry.run_all(&()).await.unwrap_err();
        assert!(matches!(
            error,
            RegistryError::PhaseFailed {
                phase: "migration",
                context_id: "identity",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn bootstrap_failure_stops_registry() {
        let registry = BootstrapRegistry::new(vec![BootstrapRegistration::new(
            ContextDescriptor {
                id: "identity",
                depends_on: &[],
            },
            |_platform: &()| async move { Err::<(), _>(std::io::Error::other("boom")) },
        )])
        .unwrap();

        let error = registry.bootstrap_all(&()).await.unwrap_err();
        assert!(matches!(
            error,
            RegistryError::PhaseFailed {
                phase: "bootstrap",
                context_id: "identity",
                ..
            }
        ));
    }
}
