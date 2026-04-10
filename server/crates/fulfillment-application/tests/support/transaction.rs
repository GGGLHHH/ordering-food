use async_trait::async_trait;
use ordering_food_fulfillment_application::{
    ApplicationError, TransactionContext, TransactionManager,
};
use std::{any::Any, sync::Mutex};

#[derive(Default)]
struct FakeTransactionContext;

impl TransactionContext for FakeTransactionContext {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

#[derive(Default)]
pub struct RecordingTransactionManager {
    began: Mutex<u32>,
    committed: Mutex<u32>,
    rolled_back: Mutex<u32>,
}

impl RecordingTransactionManager {
    pub fn began(&self) -> u32 {
        *self.began.lock().unwrap()
    }

    pub fn committed(&self) -> u32 {
        *self.committed.lock().unwrap()
    }

    pub fn rolled_back(&self) -> u32 {
        *self.rolled_back.lock().unwrap()
    }
}

#[async_trait]
impl TransactionManager for RecordingTransactionManager {
    async fn begin(&self) -> Result<Box<dyn TransactionContext>, ApplicationError> {
        *self.began.lock().unwrap() += 1;
        Ok(Box::new(FakeTransactionContext))
    }

    async fn commit(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        *self.committed.lock().unwrap() += 1;
        Ok(())
    }

    async fn rollback(&self, _tx: Box<dyn TransactionContext>) -> Result<(), ApplicationError> {
        *self.rolled_back.lock().unwrap() += 1;
        Ok(())
    }
}
