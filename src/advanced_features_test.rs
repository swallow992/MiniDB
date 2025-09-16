//! Advanced features test suite
//! 
//! Comprehensive tests for the high-priority features:
//! - JOIN operations (INNER, LEFT, RIGHT, FULL)
//! - B+ Tree index system
//! - Basic transaction support

#[cfg(test)]
mod tests {
    use crate::engine::Database;
    use crate::engine::table::Table;
    use crate::engine::transaction::{TransactionManager, TransactionOperation, LockType, IsolationLevel};
    use crate::sql::{parse_sql, Statement};
    use crate::sql::parser::{FromClause, JoinType};
    use crate::storage::index::{BPlusTreeIndex, Index, IndexKey, RecordId};
    use crate::types::{DataType, Schema, Tuple, Value, ColumnDefinition};
    use tempfile::TempDir;

    fn setup_test_database() -> (Database, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = Database::new(temp_dir.path()).expect("Failed to create database");
        (db, temp_dir)
    }

    fn create_test_schema() -> Schema {
        Schema {
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default: None,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Varchar(50),
                    nullable: false,
                    default: None,
                },
                ColumnDefinition {
                    name: "age".to_string(),
                    data_type: DataType::Integer,
                    nullable: true,
                    default: None,
                },
            ],
            primary_key: Some(vec![0]), // id column is primary key
        }
    }

    // ==================== JOIN OPERATION TESTS ====================

    #[test]
    fn test_join_parsing() {
        // Test parsing of different JOIN types
        let test_cases = vec![
            "SELECT * FROM users JOIN orders ON users.id = orders.user_id",
            "SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id",
            "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id",
            "SELECT * FROM users RIGHT JOIN orders ON users.id = orders.user_id",
            "SELECT * FROM users FULL OUTER JOIN orders ON users.id = orders.user_id",
        ];

        for sql in test_cases {
            let result = parse_sql(sql);
            assert!(result.is_ok(), "Failed to parse JOIN SQL: {}", sql);
            
            if let Ok(Statement::Select { from_clause, .. }) = result {
                if let Some(FromClause::Join { join_type, .. }) = from_clause {
                    // Verify that JOIN type is properly parsed
                    match sql {
                        s if s.contains("INNER JOIN") => assert_eq!(join_type, JoinType::Inner),
                        s if s.contains("LEFT JOIN") => assert_eq!(join_type, JoinType::Left),
                        s if s.contains("RIGHT JOIN") => assert_eq!(join_type, JoinType::Right),
                        s if s.contains("FULL OUTER JOIN") => assert_eq!(join_type, JoinType::Full),
                        s if s.contains("JOIN") => assert_eq!(join_type, JoinType::Inner), // Default JOIN
                        _ => panic!("Unexpected JOIN type"),
                    }
                } else {
                    panic!("Expected JOIN clause in FROM");
                }
            } else {
                panic!("Expected SELECT statement with FROM clause");
            }
        }
    }

    #[test]
    fn test_join_semantic_analysis() {
        use crate::sql::analyzer::{MemoryCatalog, SemanticAnalyzer};
        
        let mut catalog = MemoryCatalog::new();
        
        // Setup test schemas
        let users_schema = Schema {
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default: None,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Varchar(50),
                    nullable: false,
                    default: None,
                },
            ],
            primary_key: Some(vec![0]), // id column
        };
        
        let orders_schema = Schema {
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default: None,
                },
                ColumnDefinition {
                    name: "user_id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default: None,
                },
            ],
            primary_key: Some(vec![0]), // id column
        };
        
        catalog.add_table("users".to_string(), users_schema);
        catalog.add_table("orders".to_string(), orders_schema);
        
        let analyzer = SemanticAnalyzer::new(&catalog);
        
        // Test valid JOIN
        let sql = "SELECT users.name, orders.id FROM users JOIN orders ON users.id = orders.user_id";
        let stmt = parse_sql(sql).expect("Failed to parse SQL");
        let result = analyzer.analyze(stmt);
        assert!(result.is_ok(), "Valid JOIN should pass semantic analysis");
        
        // Test invalid table in JOIN
        let sql_invalid = "SELECT * FROM users JOIN nonexistent ON users.id = nonexistent.id";
        let stmt_invalid = parse_sql(sql_invalid).expect("Failed to parse SQL");
        let result_invalid = analyzer.analyze(stmt_invalid);
        assert!(result_invalid.is_err(), "JOIN with nonexistent table should fail");
    }

    #[test]
    fn test_join_execution_plan() {
        use crate::sql::planner::QueryPlanner;
        use crate::sql::analyzer::MemoryCatalog;
        
        let mut catalog = MemoryCatalog::new();
        let planner = QueryPlanner::new();
        
        // Setup catalog with test tables
        let users_schema = create_test_schema();
        catalog.add_table("users".to_string(), users_schema);
        catalog.add_table("orders".to_string(), create_test_schema());
        
        let sql = "SELECT * FROM users JOIN orders ON users.id = orders.user_id";
        let stmt = parse_sql(sql).expect("Failed to parse SQL");
        
        // Create a dummy analyzed statement for testing
        use crate::sql::analyzer::SemanticAnalyzer;
        
        let analyzer = SemanticAnalyzer::new(&catalog);
        let analyzed = analyzer.analyze(stmt).expect("Analysis should succeed");
        
        let plan = planner.create_plan(analyzed);
        assert!(plan.is_ok(), "JOIN execution plan should be generated successfully");
    }

    // ==================== B+ TREE INDEX TESTS ====================

    #[test]
    fn test_bplus_tree_basic_operations() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Integer]);
        
        // Test insertion
        let key1 = IndexKey::new(vec![Value::Integer(1)]);
        let key2 = IndexKey::new(vec![Value::Integer(2)]);
        let key3 = IndexKey::new(vec![Value::Integer(3)]);
        
        let rid1 = RecordId { page_id: 1, slot_id: 0 };
        let rid2 = RecordId { page_id: 1, slot_id: 1 };
        let rid3 = RecordId { page_id: 2, slot_id: 0 };
        
        assert!(index.insert(key1.clone(), rid1).is_ok());
        assert!(index.insert(key2.clone(), rid2).is_ok());
        assert!(index.insert(key3.clone(), rid3).is_ok());
        
        // Test search
        assert_eq!(index.search(&key1).unwrap(), Some(rid1));
        assert_eq!(index.search(&key2).unwrap(), Some(rid2));
        assert_eq!(index.search(&key3).unwrap(), Some(rid3));
        
        // Test non-existent key
        let key_missing = IndexKey::new(vec![Value::Integer(999)]);
        assert_eq!(index.search(&key_missing).unwrap(), None);
        
        // Test deletion
        assert!(index.delete(&key2).unwrap());
        assert_eq!(index.search(&key2).unwrap(), None);
        
        // Test size
        assert_eq!(index.size(), 2);
    }

    #[test]
    fn test_bplus_tree_range_scan() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Integer]);
        
        // Insert test data
        for i in 1..=10 {
            let key = IndexKey::new(vec![Value::Integer(i)]);
            let rid = RecordId { page_id: i as u32, slot_id: 0 };
            index.insert(key, rid).unwrap();
        }
        
        // Range scan from 3 to 7
        let start_key = IndexKey::new(vec![Value::Integer(3)]);
        let end_key = IndexKey::new(vec![Value::Integer(7)]);
        
        let mut iterator = index.range_scan(Some(&start_key), Some(&end_key)).unwrap();
        let mut count = 0;
        let mut current_value = 3;
        
        while let Some(entry) = iterator.next() {
            assert_eq!(entry.key.values()[0], Value::Integer(current_value));
            current_value += 1;
            count += 1;
        }
        
        assert_eq!(count, 5); // Keys 3, 4, 5, 6, 7
    }

    #[test]
    fn test_bplus_tree_compound_keys() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Varchar(50), DataType::Integer]);
        
        let key1 = IndexKey::new(vec![
            Value::Varchar("alice".to_string()),
            Value::Integer(25),
        ]);
        let key2 = IndexKey::new(vec![
            Value::Varchar("bob".to_string()),
            Value::Integer(30),
        ]);
        
        let rid1 = RecordId { page_id: 1, slot_id: 0 };
        let rid2 = RecordId { page_id: 1, slot_id: 1 };
        
        assert!(index.insert(key1.clone(), rid1).is_ok());
        assert!(index.insert(key2.clone(), rid2).is_ok());
        
        assert_eq!(index.search(&key1).unwrap(), Some(rid1));
        assert_eq!(index.search(&key2).unwrap(), Some(rid2));
    }

    #[test]
    fn test_table_with_indices() {
        let schema = create_test_schema();
        let mut table = Table::new(1, "test_table".to_string(), schema);
        
        // Create primary index on id column
        let result = table.create_primary_index(vec!["id".to_string()]);
        assert!(result.is_ok(), "Should be able to create primary index");
        
        // Create secondary index on name column
        let result = table.create_index(
            "idx_name".to_string(),
            vec!["name".to_string()],
            false,
        );
        assert!(result.is_ok(), "Should be able to create secondary index");
        
        // Test tuple insertion with index updates
        let tuple = Tuple {
            values: vec![
                Value::Integer(1),
                Value::Varchar("John".to_string()),
                Value::Integer(25),
            ],
        };
        let record_id = RecordId { page_id: 1, slot_id: 0 };
        
        let result = table.insert_with_indices(&tuple, record_id);
        assert!(result.is_ok(), "Should be able to insert tuple with index updates");
        
        // Verify indices were updated
        if let Some(primary_index) = table.get_index("PRIMARY") {
            let key = IndexKey::new(vec![Value::Integer(1)]);
            assert_eq!(primary_index.search(&key).unwrap(), Some(record_id));
        }
        
        if let Some(name_index) = table.get_index("idx_name") {
            let key = IndexKey::new(vec![Value::Varchar("John".to_string())]);
            assert_eq!(name_index.search(&key).unwrap(), Some(record_id));
        }
    }

    // ==================== TRANSACTION TESTS ====================

    #[test]
    fn test_transaction_basic_lifecycle() {
        let tm = TransactionManager::new();
        
        // Begin transaction
        let txn_id = tm.begin_transaction().unwrap();
        assert!(tm.get_transaction_state(txn_id).is_some());
        
        // Log some operations
        let insert_op = TransactionOperation::Insert {
            table: "users".to_string(),
            record_id: "1".to_string(),
        };
        tm.log_operation(txn_id, insert_op).unwrap();
        
        let update_op = TransactionOperation::Update {
            table: "users".to_string(),
            record_id: "1".to_string(),
            old_values: vec!["John".to_string()],
            new_values: vec!["Jane".to_string()],
        };
        tm.log_operation(txn_id, update_op).unwrap();
        
        // Commit transaction
        tm.commit_transaction(txn_id).unwrap();
        
        use crate::engine::transaction::TransactionState;
        assert_eq!(tm.get_transaction_state(txn_id), Some(TransactionState::Committed));
    }

    #[test]
    fn test_transaction_rollback() {
        let tm = TransactionManager::new();
        
        let txn_id = tm.begin_transaction().unwrap();
        
        // Log operations
        let insert_op = TransactionOperation::Insert {
            table: "users".to_string(),
            record_id: "1".to_string(),
        };
        tm.log_operation(txn_id, insert_op).unwrap();
        
        let delete_op = TransactionOperation::Delete {
            table: "users".to_string(),
            record_id: "2".to_string(),
            old_values: vec!["Bob".to_string(), "30".to_string()],
        };
        tm.log_operation(txn_id, delete_op).unwrap();
        
        // Rollback
        tm.rollback_transaction(txn_id).unwrap();
        
        use crate::engine::transaction::TransactionState;
        assert_eq!(tm.get_transaction_state(txn_id), Some(TransactionState::Aborted));
    }

    #[test]
    fn test_transaction_isolation_levels() {
        let tm = TransactionManager::new();
        
        // Test different isolation levels
        let txn1 = tm.begin_transaction_with_isolation(IsolationLevel::ReadCommitted).unwrap();
        let txn2 = tm.begin_transaction_with_isolation(IsolationLevel::Serializable).unwrap();
        
        use crate::engine::transaction::TransactionState;
        assert_eq!(tm.get_transaction_state(txn1), Some(TransactionState::Active));
        assert_eq!(tm.get_transaction_state(txn2), Some(TransactionState::Active));
        
        tm.commit_transaction(txn1).unwrap();
        tm.commit_transaction(txn2).unwrap();
    }

    #[test]
    fn test_transaction_locking() {
        let tm = TransactionManager::new();
        
        let txn1 = tm.begin_transaction().unwrap();
        let txn2 = tm.begin_transaction().unwrap();
        
        // First transaction acquires read lock
        let result = tm.acquire_lock(txn1, "table1".to_string(), LockType::SharedRead);
        assert!(result.is_ok(), "Should acquire read lock successfully");
        
        // Second transaction also acquires read lock (should succeed)
        let result = tm.acquire_lock(txn2, "table1".to_string(), LockType::SharedRead);
        assert!(result.is_ok(), "Multiple read locks should be allowed");
        
        // Second transaction tries to acquire write lock (should fail)
        let result = tm.acquire_lock(txn2, "table1".to_string(), LockType::ExclusiveWrite);
        assert!(result.is_err(), "Write lock should conflict with existing read locks");
        
        tm.commit_transaction(txn1).unwrap();
        tm.commit_transaction(txn2).unwrap();
    }

    #[test]
    fn test_concurrent_transactions() {
        let tm = TransactionManager::new();
        
        // Simulate concurrent transactions
        let txn1 = tm.begin_transaction().unwrap();
        let txn2 = tm.begin_transaction().unwrap();
        let txn3 = tm.begin_transaction().unwrap();
        
        // Check active transactions
        let active = tm.list_active_transactions();
        assert_eq!(active.len(), 3);
        assert!(active.contains(&txn1));
        assert!(active.contains(&txn2));
        assert!(active.contains(&txn3));
        
        // Commit transactions in different order
        tm.commit_transaction(txn2).unwrap();
        tm.rollback_transaction(txn3).unwrap();
        tm.commit_transaction(txn1).unwrap();
        
        // Check final states
        use crate::engine::transaction::TransactionState;
        assert_eq!(tm.get_transaction_state(txn1), Some(TransactionState::Committed));
        assert_eq!(tm.get_transaction_state(txn2), Some(TransactionState::Committed));
        assert_eq!(tm.get_transaction_state(txn3), Some(TransactionState::Aborted));
    }

    // ==================== INTEGRATION TESTS ====================

    #[test]
    fn test_integrated_advanced_features() {
        let (mut db, _temp_dir) = setup_test_database();
        
        // Create tables with indices
        let create_users = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(50), age INT)";
        let create_orders = "CREATE TABLE orders (id INT PRIMARY KEY, user_id INT, amount INT)";
        
        let result1 = db.execute(create_users);
        assert!(result1.is_ok(), "Should create users table: {:?}", result1.err());
        
        let result2 = db.execute(create_orders);
        assert!(result2.is_ok(), "Should create orders table: {:?}", result2.err());
        
        // Insert test data
        let insert_users = vec![
            "INSERT INTO users VALUES (1, 'Alice', 25)",
            "INSERT INTO users VALUES (2, 'Bob', 30)",
            "INSERT INTO users VALUES (3, 'Charlie', 35)",
        ];
        
        let insert_orders = vec![
            "INSERT INTO orders VALUES (1, 1, 100)",
            "INSERT INTO orders VALUES (2, 1, 200)",
            "INSERT INTO orders VALUES (3, 2, 150)",
        ];
        
        for sql in insert_users {
            let result = db.execute(sql);
            assert!(result.is_ok(), "Should insert user: {:?}", result.err());
        }
        
        for sql in insert_orders {
            let result = db.execute(sql);
            assert!(result.is_ok(), "Should insert order: {:?}", result.err());
        }
        
        // Test advanced queries
        let advanced_queries = vec![
            "SELECT * FROM users WHERE age > 25 ORDER BY age DESC",
            "SELECT COUNT(*) FROM users",
            "SELECT AVG(age) FROM users",
            "SELECT name FROM users GROUP BY name",
        ];
        
        for sql in advanced_queries {
            let result = db.execute(sql);
            assert!(result.is_ok(), "Advanced query should succeed: {} -> {:?}", sql, result.err());
        }
    }

    #[test]
    fn test_error_handling() {
        let (mut db, _temp_dir) = setup_test_database();
        
        // Test various error conditions
        let error_cases = vec![
            ("SELECT * FROM nonexistent", "Table not found error"),
            ("INSERT INTO users VALUES (1)", "Column count mismatch"),
            ("SELECT nonexistent_column FROM users", "Column not found"),
            ("CREATE TABLE users (id INT, id INT)", "Duplicate column name"),
        ];
        
        // First create a test table
        db.execute("CREATE TABLE users (id INT, name VARCHAR(50))").unwrap();
        
        for (sql, description) in error_cases {
            let result = db.execute(sql);
            assert!(result.is_err(), "{} should fail: {}", description, sql);
        }
    }

    #[test]
    fn test_stress_operations() {
        let (mut db, _temp_dir) = setup_test_database();
        
        // Create table
        db.execute("CREATE TABLE stress_test (id INT, value INT)").unwrap();
        
        // Insert many records
        for i in 1..=100 {
            let sql = format!("INSERT INTO stress_test VALUES ({}, {})", i, i * 2);
            let result = db.execute(&sql);
            assert!(result.is_ok(), "Bulk insert should succeed for record {}", i);
        }
        
        // Test bulk operations
        let result = db.execute("SELECT COUNT(*) FROM stress_test");
        assert!(result.is_ok());
        
        let result = db.execute("SELECT AVG(value) FROM stress_test");
        assert!(result.is_ok());
        
        let result = db.execute("SELECT * FROM stress_test WHERE id > 50 ORDER BY value DESC LIMIT 10");
        assert!(result.is_ok());
    }
}