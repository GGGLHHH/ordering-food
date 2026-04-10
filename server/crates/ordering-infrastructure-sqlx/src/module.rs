use crate::{
    SqlxOrderReadRepository, SqlxOrderRepository, SqlxOutboxMessageAppender,
    SqlxTransactionManager,
};
use sqlx::PgPool;
use std::sync::Arc;

pub struct OrderingSqlxComponents {
    pub order_repository: Arc<SqlxOrderRepository>,
    pub order_read_repository: Arc<SqlxOrderReadRepository>,
    pub transaction_manager: Arc<SqlxTransactionManager>,
    pub outbox_message_appender: Arc<SqlxOutboxMessageAppender>,
}

pub fn build_ordering_sqlx_components(pool: PgPool) -> OrderingSqlxComponents {
    OrderingSqlxComponents {
        order_repository: Arc::new(SqlxOrderRepository),
        order_read_repository: Arc::new(SqlxOrderReadRepository::new(pool.clone())),
        transaction_manager: Arc::new(SqlxTransactionManager::new(pool)),
        outbox_message_appender: Arc::new(SqlxOutboxMessageAppender),
    }
}
