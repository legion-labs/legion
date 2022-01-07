use async_trait::async_trait;
use lgn_telemetry::{info, warn};

use crate::LockContext;

/// Definition of a Transaction
pub struct Transaction {
    /// Transaction Unique Identifier
    id: uuid::Uuid,
    /// List of operation within the transaction
    operations: Vec<Box<dyn TransactionOperation + Send + Sync>>,
}

#[async_trait]
pub trait TransactionOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()>;
    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()>;
}

impl Transaction {
    /// Create a new Transaction
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let new_transaction = Self {
            id: uuid::Uuid::new_v4(),
            operations: Vec::new(),
        };
        info!("Begin Transaction: {}", &new_transaction.id);
        new_transaction
    }

    pub(crate) async fn apply_transaction(
        &mut self,
        mut context: LockContext<'_>,
    ) -> anyhow::Result<()> {
        // Try to apply all the operations
        let mut rollback_state: Option<(anyhow::Error, usize)> = None;
        for (index, op) in self.operations.iter_mut().enumerate() {
            if let Err(op_err) = op.apply_operation(&mut context).await {
                rollback_state = Some((op_err, index));
                break;
            }
        }

        // If an ops failed, save the error state to rollback all the previous
        // transaction's operations
        if let Some((op_err, rollback_index)) = rollback_state {
            warn!("Transaction {} failed to commit: {}", &self.id, op_err);
            for op in self.operations.iter().take(rollback_index).rev() {
                op.rollback_operation(&mut context)
                    .await
                    .unwrap_or_else(|op_err| warn!("\tfailed to rollback ops: {}", op_err));
            }
            Err(op_err)
        } else {
            // All the ops complete, the the resources
            context.save_changed_resources().await?;
            info!(
                "Transaction Applied: {} / {}ops",
                &self.id,
                self.operations.len()
            );
            Ok(())
        }
    }

    pub(crate) async fn rollback_transaction(
        &mut self,
        mut context: LockContext<'_>,
    ) -> anyhow::Result<()> {
        // Try to rollback all transaction operations (in reverse order)
        let mut rollback_state: Option<(anyhow::Error, usize)> = None;
        for (index, op) in self.operations.iter().rev().enumerate() {
            if let Err(op_err) = op.rollback_operation(&mut context).await {
                // If the rollback failed, abort rollback
                rollback_state = Some((op_err, index));
                break;
            }
        }

        // If the rollback failed, reapply the previous operations that pass
        if let Some((op_err, rollback_index)) = rollback_state {
            warn!("Transaction {} failed to rollback: {}", &self.id, op_err);
            for op in self.operations.iter_mut().rev().take(rollback_index).rev() {
                op.apply_operation(&mut context)
                    .await
                    .unwrap_or_else(|op_err| warn!("\tfailed to reapply ops: {}", op_err));
            }
            Err(op_err)
        } else {
            // All the ops complete, the the resources
            context.save_changed_resources().await?;
            info!(
                "Transaction Rollbacked: {} / {}ops",
                &self.id,
                self.operations.len()
            );
            Ok(())
        }
    }

    /// Add a new operation to the transaction
    pub fn add_operation(mut self, op: Box<dyn TransactionOperation + Send + Sync>) -> Self {
        self.operations.push(op);
        self
    }
}
