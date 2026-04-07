mod error;
mod module;
mod ordering_event_handler;
mod ports;
pub mod use_cases;

pub use error::ApplicationError;
pub use module::FulfillmentModule;
pub use ordering_event_handler::OrderingCommercialEventHandler;
pub use ports::{
    Clock, CommercialOrderProjectionItemReadModel, CommercialOrderProjectionQueryService,
    CommercialOrderProjectionReadModel, CommercialOrderProjectionReadRepository,
    CommercialOrderProjectionStore, IdGenerator, OrderCancelledByCustomer,
    OrderCommercialStateChanged, OrderPlaced, OrderPlacedItem, OutboxMessage, OutboxMessageReader,
    ProjectionCheckpoint, ProjectionCheckpointStore, TransactionContext, TransactionManager,
    WorkflowOrderQueryService, WorkflowOrderReadModel, WorkflowOrderReadRepository,
    WorkflowOrderRepository,
};
pub use use_cases::{
    AcceptOrder, AcceptOrderInput, CompleteOrder, CompleteOrderInput, MarkOrderReadyForPickup,
    MarkOrderReadyForPickupInput, RejectOrderByStore, RejectOrderByStoreInput, StartPreparingOrder,
    StartPreparingOrderInput,
};
