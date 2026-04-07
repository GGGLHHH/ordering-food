use crate::{
    display_rule_as_str, parse_uuid, sellable_status_as_str, transaction::SqlxTransactionContext,
};
use async_trait::async_trait;
use ordering_food_catalog_application::{
    ApplicationError, StoreItemListingRepository, TransactionContext,
};
use ordering_food_catalog_domain::StoreItemListing;
use sqlx::{Postgres, Transaction};

#[derive(Debug, Default)]
pub struct SqlxStoreItemListingRepository;

impl SqlxStoreItemListingRepository {
    fn transaction(
        tx: &mut dyn TransactionContext,
    ) -> Result<&mut Transaction<'static, Postgres>, ApplicationError> {
        tx.as_any_mut()
            .downcast_mut::<SqlxTransactionContext>()
            .map(SqlxTransactionContext::transaction_mut)
            .ok_or_else(|| {
                ApplicationError::unexpected("unexpected transaction context implementation")
            })
    }
}

#[async_trait]
impl StoreItemListingRepository for SqlxStoreItemListingRepository {
    async fn upsert(
        &self,
        tx: &mut dyn TransactionContext,
        listing: &StoreItemListing,
    ) -> Result<(), ApplicationError> {
        sqlx::query(
            r#"
            INSERT INTO catalog.store_item_listings (
                store_catalog_id,
                item_id,
                price_amount,
                status,
                display_rule,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (store_catalog_id, item_id)
            DO UPDATE SET
                price_amount = EXCLUDED.price_amount,
                status = EXCLUDED.status,
                display_rule = EXCLUDED.display_rule,
                updated_at = NOW()
            "#,
        )
        .bind(parse_uuid(
            listing.store_catalog_id().as_str(),
            "store catalog id",
        )?)
        .bind(parse_uuid(listing.item_id().as_str(), "item id")?)
        .bind(listing.price().amount())
        .bind(sellable_status_as_str(listing.status()))
        .bind(display_rule_as_str(listing.display_rule()))
        .execute(&mut **Self::transaction(tx)?)
        .await
        .map_err(|error| {
            ApplicationError::unexpected_with_source("failed to upsert store item listing", error)
        })?;

        Ok(())
    }
}
