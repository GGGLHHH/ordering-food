use crate::{ContextDescriptor, RegistryError};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub struct ContextOrderPlanner;

impl ContextOrderPlanner {
    pub fn plan(
        descriptors: impl IntoIterator<Item = ContextDescriptor>,
    ) -> Result<Vec<ContextDescriptor>, RegistryError> {
        let descriptors = descriptors.into_iter().collect::<Vec<_>>();
        let mut descriptor_by_id = BTreeMap::new();

        for descriptor in &descriptors {
            if descriptor_by_id
                .insert(descriptor.id, *descriptor)
                .is_some()
            {
                return Err(RegistryError::DuplicateContextId {
                    context_id: descriptor.id,
                });
            }
        }

        let mut indegree_by_id = descriptor_by_id
            .keys()
            .copied()
            .map(|id| (id, 0usize))
            .collect::<BTreeMap<_, _>>();
        let mut dependents_by_id = descriptor_by_id
            .keys()
            .copied()
            .map(|id| (id, Vec::new()))
            .collect::<BTreeMap<_, Vec<&'static str>>>();

        for descriptor in descriptor_by_id.values() {
            for dependency_id in descriptor.depends_on {
                if !descriptor_by_id.contains_key(dependency_id) {
                    return Err(RegistryError::MissingDependency {
                        context_id: descriptor.id,
                        dependency_id,
                    });
                }

                *indegree_by_id
                    .get_mut(descriptor.id)
                    .expect("descriptor indegree missing") += 1;
                dependents_by_id
                    .get_mut(dependency_id)
                    .expect("dependent list missing")
                    .push(descriptor.id);
            }
        }

        let mut ready = indegree_by_id
            .iter()
            .filter_map(|(id, indegree)| (*indegree == 0).then_some(*id))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<VecDeque<_>>();
        let mut ordered = Vec::with_capacity(descriptor_by_id.len());

        while let Some(context_id) = ready.pop_front() {
            ordered.push(
                *descriptor_by_id
                    .get(context_id)
                    .expect("descriptor should exist during planning"),
            );

            if let Some(dependents) = dependents_by_id.get(context_id) {
                let mut newly_ready = BTreeSet::new();
                for dependent_id in dependents {
                    let indegree = indegree_by_id
                        .get_mut(dependent_id)
                        .expect("dependent indegree missing");
                    *indegree -= 1;
                    if *indegree == 0 {
                        newly_ready.insert(*dependent_id);
                    }
                }
                ready.extend(newly_ready);
            }
        }

        if ordered.len() != descriptor_by_id.len() {
            let remaining = indegree_by_id
                .into_iter()
                .filter_map(|(context_id, indegree)| (indegree > 0).then_some(context_id))
                .collect::<Vec<_>>();
            return Err(RegistryError::CyclicDependency {
                context_ids: remaining,
            });
        }

        Ok(ordered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_context_id() {
        let error = ContextOrderPlanner::plan([
            ContextDescriptor {
                id: "identity",
                depends_on: &[],
            },
            ContextDescriptor {
                id: "identity",
                depends_on: &[],
            },
        ])
        .unwrap_err();

        assert!(matches!(
            error,
            RegistryError::DuplicateContextId {
                context_id: "identity"
            }
        ));
    }

    #[test]
    fn rejects_missing_dependency() {
        let error = ContextOrderPlanner::plan([ContextDescriptor {
            id: "ordering",
            depends_on: &["identity"],
        }])
        .unwrap_err();

        assert!(matches!(
            error,
            RegistryError::MissingDependency {
                context_id: "ordering",
                dependency_id: "identity"
            }
        ));
    }

    #[test]
    fn rejects_cyclic_dependency() {
        let error = ContextOrderPlanner::plan([
            ContextDescriptor {
                id: "identity",
                depends_on: &["ordering"],
            },
            ContextDescriptor {
                id: "ordering",
                depends_on: &["identity"],
            },
        ])
        .unwrap_err();

        assert!(matches!(error, RegistryError::CyclicDependency { .. }));
    }

    #[test]
    fn orders_dependencies_topologically() {
        let ordered = ContextOrderPlanner::plan([
            ContextDescriptor {
                id: "ordering",
                depends_on: &["identity"],
            },
            ContextDescriptor {
                id: "identity",
                depends_on: &[],
            },
        ])
        .unwrap();

        assert_eq!(ordered[0].id, "identity");
        assert_eq!(ordered[1].id, "ordering");
    }
}
