use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError as CatalogApplicationError, CatalogBrandScope, CatalogStoreScope,
    OrganizationScopeReader,
};
use ordering_food_organization_published::{
    BrandLookupGateway, BrandRef, OrganizationCollaborationError, StoreScopeGateway, StoreSummary,
};
use std::sync::Arc;

pub(crate) struct CatalogOrganizationScopeAclAdapter {
    brand_lookup_gateway: Arc<dyn BrandLookupGateway>,
    store_scope_gateway: Arc<dyn StoreScopeGateway>,
}

impl CatalogOrganizationScopeAclAdapter {
    pub(crate) fn new(
        brand_lookup_gateway: Arc<dyn BrandLookupGateway>,
        store_scope_gateway: Arc<dyn StoreScopeGateway>,
    ) -> Self {
        Self {
            brand_lookup_gateway,
            store_scope_gateway,
        }
    }
}

#[async_trait]
impl OrganizationScopeReader for CatalogOrganizationScopeAclAdapter {
    async fn get_active_store(&self) -> Result<Option<CatalogStoreScope>, CatalogApplicationError> {
        let store = self
            .store_scope_gateway
            .get_active()
            .await
            .map_err(map_organization_error)?;

        Ok(store.map(map_store_summary))
    }

    async fn get_brand(
        &self,
        brand_id: &str,
    ) -> Result<Option<CatalogBrandScope>, CatalogApplicationError> {
        let brand = self
            .brand_lookup_gateway
            .get_by_id(brand_id)
            .await
            .map_err(map_organization_error)?;

        Ok(brand.map(map_brand_ref))
    }

    async fn get_store_scope(
        &self,
        brand_id: &str,
        store_id: &str,
    ) -> Result<Option<CatalogStoreScope>, CatalogApplicationError> {
        let store = self
            .store_scope_gateway
            .get_by_id(store_id)
            .await
            .map_err(map_organization_error)?;
        let store_scope = store.map(map_store_summary);

        Ok(store_scope.filter(|store| store.brand_id == brand_id))
    }
}

fn map_brand_ref(brand: BrandRef) -> CatalogBrandScope {
    CatalogBrandScope {
        brand_id: brand.brand_id,
    }
}

fn map_store_summary(store: StoreSummary) -> CatalogStoreScope {
    CatalogStoreScope {
        store_id: store.store_id,
        brand_id: store.brand_id,
        slug: store.slug,
        name: store.name,
        currency_code: store.currency_code,
        timezone: store.timezone,
        status: store.status,
    }
}

fn map_organization_error(error: OrganizationCollaborationError) -> CatalogApplicationError {
    match error {
        OrganizationCollaborationError::Validation { message } => {
            CatalogApplicationError::validation(message)
        }
        OrganizationCollaborationError::NotFound { message } => {
            CatalogApplicationError::not_found(message)
        }
        OrganizationCollaborationError::Conflict { message } => {
            CatalogApplicationError::conflict(message)
        }
        OrganizationCollaborationError::Unexpected { message, details } => match details {
            Some(source) => CatalogApplicationError::Unexpected {
                message,
                source: Some(Box::new(std::io::Error::other(source))),
            },
            None => CatalogApplicationError::unexpected(message),
        },
    }
}
