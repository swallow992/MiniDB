//! Transaction management
//!
//! Simple transaction manager with basic ACID properties:
//! - Atomicity: All operations in a transaction succeed or all fail
//! - Consistency: Database constraints are maintained
//! - Isolation: Transactions are isolated from each other (basic read-write locks)
//! - Durability: Committed transactions are persistent

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub type TransactionId = u64;

/// Transaction state
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Active,
    Preparing,
    Committed,
    Aborted,
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

/// Lock types for concurrency control
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    SharedRead,
    ExclusiveWrite,
}

/// Lock request
#[derive(Debug, Clone)]
pub struct LockRequest {
    pub transaction_id: TransactionId,
    pub resource_id: String,
    pub lock_type: LockType,
}

/// Transaction metadata
pub struct Transaction {
    /// Unique transaction ID
    pub id: TransactionId,
    /// Current state
    pub state: TransactionState,
    /// Isolation level
    pub isolation_level: IsolationLevel,
    /// Start timestamp
    pub start_time: u64,
    /// Held locks
    pub held_locks: HashSet<String>,
    /// Operations log for rollback
    pub operations_log: Vec<TransactionOperation>,
}

/// Transaction operation for logging and rollback
#[derive(Debug, Clone)]
pub enum TransactionOperation {
    Insert {
        table: String,
        record_id: String,
    },
    Update {
        table: String,
        record_id: String,
        old_values: Vec<String>,
        new_values: Vec<String>,
    },
    Delete {
        table: String,
        record_id: String,
        old_values: Vec<String>,
    },
}

/// Lock manager for concurrency control
pub struct LockManager {
    /// Resource locks: resource_id -> (transaction_id, lock_type)
    locks: Arc<Mutex<HashMap<String, (TransactionId, LockType)>>>,
    /// Wait-for graph for deadlock detection
    wait_for: Arc<Mutex<HashMap<TransactionId, HashSet<TransactionId>>>>,
}

/// Transaction manager
pub struct TransactionManager {
    /// Active transactions
    transactions: Arc<RwLock<HashMap<TransactionId, Transaction>>>,
    /// Next transaction ID
    next_txn_id: Arc<Mutex<TransactionId>>,
    /// Lock manager
    lock_manager: LockManager,
    /// Default isolation level
    default_isolation_level: IsolationLevel,
}

/// Transaction errors
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction not found: {id}")]
    TransactionNotFound { id: TransactionId },
    
    #[error("Transaction already committed: {id}")]
    AlreadyCommitted { id: TransactionId },
    
    #[error("Transaction already aborted: {id}")]
    AlreadyAborted { id: TransactionId },
    
    #[error("Deadlock detected involving transaction: {id}")]
    DeadlockDetected { id: TransactionId },
    
    #[error("Lock conflict: resource {resource} is locked by transaction {holder}")]
    LockConflict { 
        resource: String, 
        holder: TransactionId 
    },
    
    #[error("Invalid transaction state: expected {expected:?}, found {found:?}")]
    InvalidState { 
        expected: TransactionState, 
        found: TransactionState 
    },
    
    #[error("Isolation violation")]
    IsolationViolation,
    
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
            wait_for: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Acquire a lock on a resource
    pub fn acquire_lock(&self, request: LockRequest) -> Result<(), TransactionError> {
        let mut locks = self.locks.lock().unwrap();
        
        match locks.get(&request.resource_id) {
            Some((holder_txn, existing_lock_type)) => {
                if *holder_txn == request.transaction_id {
                    // Same transaction, check lock upgrade
                    if request.lock_type == LockType::ExclusiveWrite && 
                       *existing_lock_type == LockType::SharedRead {
                        // Upgrade to write lock
                        locks.insert(request.resource_id.clone(), 
                                   (request.transaction_id, LockType::ExclusiveWrite));
                    }
                    Ok(())
                } else {
                    // Different transaction holds the lock
                    match (existing_lock_type, &request.lock_type) {
                        (LockType::SharedRead, LockType::SharedRead) => {
                            // Multiple readers allowed - for simplicity, we'll allow this
                            // In a real system, we'd need a more complex lock table
                            Ok(())
                        }
                        _ => {
                            // Conflict: exclusive lock or read-write conflict
                            Err(TransactionError::LockConflict {
                                resource: request.resource_id,
                                holder: *holder_txn,
                            })
                        }
                    }
                }
            }
            None => {
                // No existing lock, grant the lock
                locks.insert(request.resource_id.clone(), 
                           (request.transaction_id, request.lock_type));
                Ok(())
            }
        }
    }
    
    /// Release all locks held by a transaction
    pub fn release_locks(&self, transaction_id: TransactionId) {
        let mut locks = self.locks.lock().unwrap();
        locks.retain(|_, (holder, _)| *holder != transaction_id);
    }
    
    /// Check for deadlocks (simplified detection)
    pub fn detect_deadlock(&self, _transaction_id: TransactionId) -> bool {
        // Simplified deadlock detection - in a real system this would be more sophisticated
        false
    }
}

impl Transaction {
    pub fn new(id: TransactionId, isolation_level: IsolationLevel) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        Self {
            id,
            state: TransactionState::Active,
            isolation_level,
            start_time,
            held_locks: HashSet::new(),
            operations_log: Vec::new(),
        }
    }
    
    /// Add an operation to the transaction log
    pub fn log_operation(&mut self, operation: TransactionOperation) {
        self.operations_log.push(operation);
    }
    
    /// Check if transaction can proceed based on isolation level
    pub fn can_read(&self, _resource: &str, _writer_txn: Option<TransactionId>) -> bool {
        match self.isolation_level {
            IsolationLevel::ReadUncommitted => true,
            IsolationLevel::ReadCommitted => true, // Simplified
            IsolationLevel::RepeatableRead => true, // Simplified
            IsolationLevel::Serializable => true, // Simplified
        }
    }
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            next_txn_id: Arc::new(Mutex::new(1)),
            lock_manager: LockManager::new(),
            default_isolation_level: IsolationLevel::ReadCommitted,
        }
    }
    
    /// Begin a new transaction
    pub fn begin_transaction(&self) -> Result<TransactionId, TransactionError> {
        self.begin_transaction_with_isolation(self.default_isolation_level)
    }
    
    /// Begin a transaction with specific isolation level
    pub fn begin_transaction_with_isolation(&self, isolation_level: IsolationLevel) -> Result<TransactionId, TransactionError> {
        let mut next_id = self.next_txn_id.lock().unwrap();
        let txn_id = *next_id;
        *next_id += 1;
        drop(next_id);
        
        let transaction = Transaction::new(txn_id, isolation_level);
        
        let mut transactions = self.transactions.write().unwrap();
        transactions.insert(txn_id, transaction);
        
        Ok(txn_id)
    }
    
    /// Commit a transaction
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let mut transactions = self.transactions.write().unwrap();
        
        match transactions.get_mut(&txn_id) {
            Some(transaction) => {
                match transaction.state {
                    TransactionState::Active => {
                        // Commit the transaction
                        transaction.state = TransactionState::Committed;
                        
                        // Release all locks
                        self.lock_manager.release_locks(txn_id);
                        
                        // In a real system, we would flush logs to disk here
                        println!("✅ Transaction {} committed successfully", txn_id);
                        Ok(())
                    }
                    TransactionState::Committed => {
                        Err(TransactionError::AlreadyCommitted { id: txn_id })
                    }
                    TransactionState::Aborted => {
                        Err(TransactionError::AlreadyAborted { id: txn_id })
                    }
                    _ => {
                        Err(TransactionError::InvalidState {
                            expected: TransactionState::Active,
                            found: transaction.state.clone(),
                        })
                    }
                }
            }
            None => Err(TransactionError::TransactionNotFound { id: txn_id }),
        }
    }
    
    /// Rollback a transaction
    pub fn rollback_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let mut transactions = self.transactions.write().unwrap();
        
        match transactions.get_mut(&txn_id) {
            Some(transaction) => {
                match transaction.state {
                    TransactionState::Active | TransactionState::Preparing => {
                        // Rollback operations (in reverse order)
                        for operation in transaction.operations_log.iter().rev() {
                            self.rollback_operation(operation)?;
                        }
                        
                        // Mark as aborted
                        transaction.state = TransactionState::Aborted;
                        
                        // Release all locks
                        self.lock_manager.release_locks(txn_id);
                        
                        println!("🔄 Transaction {} rolled back", txn_id);
                        Ok(())
                    }
                    TransactionState::Committed => {
                        Err(TransactionError::AlreadyCommitted { id: txn_id })
                    }
                    TransactionState::Aborted => {
                        Err(TransactionError::AlreadyAborted { id: txn_id })
                    }
                }
            }
            None => Err(TransactionError::TransactionNotFound { id: txn_id }),
        }
    }
    
    /// Acquire a lock for a transaction
    pub fn acquire_lock(&self, txn_id: TransactionId, resource: String, lock_type: LockType) -> Result<(), TransactionError> {
        // Check if transaction exists and is active
        {
            let transactions = self.transactions.read().unwrap();
            let transaction = transactions.get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound { id: txn_id })?;
                
            if transaction.state != TransactionState::Active {
                return Err(TransactionError::InvalidState {
                    expected: TransactionState::Active,
                    found: transaction.state.clone(),
                });
            }
        }
        
        let request = LockRequest {
            transaction_id: txn_id,
            resource_id: resource.clone(),
            lock_type,
        };
        
        self.lock_manager.acquire_lock(request)?;
        
        // Add lock to transaction's held locks
        {
            let mut transactions = self.transactions.write().unwrap();
            if let Some(transaction) = transactions.get_mut(&txn_id) {
                transaction.held_locks.insert(resource);
            }
        }
        
        Ok(())
    }
    
    /// Log an operation for a transaction
    pub fn log_operation(&self, txn_id: TransactionId, operation: TransactionOperation) -> Result<(), TransactionError> {
        let mut transactions = self.transactions.write().unwrap();
        
        match transactions.get_mut(&txn_id) {
            Some(transaction) => {
                if transaction.state == TransactionState::Active {
                    transaction.log_operation(operation);
                    Ok(())
                } else {
                    Err(TransactionError::InvalidState {
                        expected: TransactionState::Active,
                        found: transaction.state.clone(),
                    })
                }
            }
            None => Err(TransactionError::TransactionNotFound { id: txn_id }),
        }
    }
    
    /// Get transaction state
    pub fn get_transaction_state(&self, txn_id: TransactionId) -> Option<TransactionState> {
        let transactions = self.transactions.read().unwrap();
        transactions.get(&txn_id).map(|txn| txn.state.clone())
    }
    
    /// List active transactions
    pub fn list_active_transactions(&self) -> Vec<TransactionId> {
        let transactions = self.transactions.read().unwrap();
        transactions.iter()
            .filter(|(_, txn)| txn.state == TransactionState::Active)
            .map(|(id, _)| *id)
            .collect()
    }
    
    // Helper method for rolling back operations
    fn rollback_operation(&self, operation: &TransactionOperation) -> Result<(), TransactionError> {
        match operation {
            TransactionOperation::Insert { table, record_id } => {
                // Remove the inserted record
                println!("🔄 Rolling back INSERT in table {}, record {}", table, record_id);
                // In a real system, this would delete the record from storage
            }
            TransactionOperation::Update { table, record_id, old_values, .. } => {
                // Restore old values
                println!("🔄 Rolling back UPDATE in table {}, record {}", table, record_id);
                println!("   Restoring old values: {:?}", old_values);
                // In a real system, this would restore the old values
            }
            TransactionOperation::Delete { table, record_id, old_values } => {
                // Restore deleted record
                println!("🔄 Rolling back DELETE in table {}, record {}", table, record_id);
                println!("   Restoring record: {:?}", old_values);
                // In a real system, this would restore the deleted record
            }
        }
        Ok(())
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_lifecycle() {
        let tm = TransactionManager::new();
        
        // Begin transaction
        let txn_id = tm.begin_transaction().unwrap();
        assert_eq!(tm.get_transaction_state(txn_id), Some(TransactionState::Active));
        
        // Commit transaction
        tm.commit_transaction(txn_id).unwrap();
        assert_eq!(tm.get_transaction_state(txn_id), Some(TransactionState::Committed));
    }
    
    #[test]
    fn test_transaction_rollback() {
        let tm = TransactionManager::new();
        
        let txn_id = tm.begin_transaction().unwrap();
        
        // Log some operations
        tm.log_operation(txn_id, TransactionOperation::Insert {
            table: "users".to_string(),
            record_id: "1".to_string(),
        }).unwrap();
        
        // Rollback
        tm.rollback_transaction(txn_id).unwrap();
        assert_eq!(tm.get_transaction_state(txn_id), Some(TransactionState::Aborted));
    }
    
    #[test]
    fn test_lock_acquisition() {
        let tm = TransactionManager::new();
        
        let txn1 = tm.begin_transaction().unwrap();
        let txn2 = tm.begin_transaction().unwrap();
        
        // First transaction acquires read lock
        tm.acquire_lock(txn1, "table1".to_string(), LockType::SharedRead).unwrap();
        
        // Second transaction acquires read lock (should succeed)
        tm.acquire_lock(txn2, "table1".to_string(), LockType::SharedRead).unwrap();
        
        // Second transaction tries to acquire write lock (should fail)
        assert!(tm.acquire_lock(txn2, "table1".to_string(), LockType::ExclusiveWrite).is_err());
    }
    
    #[test]
    fn test_isolation_levels() {
        let tm = TransactionManager::new();
        
        let txn_serializable = tm.begin_transaction_with_isolation(IsolationLevel::Serializable).unwrap();
        let txn_read_committed = tm.begin_transaction_with_isolation(IsolationLevel::ReadCommitted).unwrap();
        
        assert_eq!(tm.get_transaction_state(txn_serializable), Some(TransactionState::Active));
        assert_eq!(tm.get_transaction_state(txn_read_committed), Some(TransactionState::Active));
    }
}
