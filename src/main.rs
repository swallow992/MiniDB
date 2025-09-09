use minidb::engine::database::QueryResult;
use minidb::Database;
use std::env;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("MiniDB v{}", minidb::VERSION);
    println!("Type 'help' for available commands or 'quit' to exit.");

    // Get database path from command line args or use default
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "./minidb_data".to_string());

    let mut database = Database::new(&db_path)?;

    loop {
        print!("minidb> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "\\q" => {
                println!("Goodbye!");
                break;
            }
            "help" | "\\h" => {
                show_help();
            }
            "\\d" => {
                show_tables(&database)?;
            }
            _ => match execute_sql(&mut database, input) {
                Ok(result) => print_result(&result),
                Err(e) => eprintln!("Error: {}", e),
            },
        }
    }

    Ok(())
}

fn show_help() {
    println!("Available commands:");
    println!("  help, \\h          Show this help message");
    println!("  quit, exit, \\q    Exit the program");
    println!("  \\d                List all tables");
    println!();
    println!("SQL commands:");
    println!("  CREATE TABLE name (column_definitions...)");
    println!("  INSERT INTO name VALUES (...)");
    println!("  SELECT columns FROM name [WHERE condition]");
    println!("  UPDATE name SET column=value [WHERE condition]");
    println!("  DELETE FROM name [WHERE condition]");
    println!("  DROP TABLE name");
    println!();
}

fn show_tables(_database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("Tables in database:");
    // TODO: Implement table listing functionality
    println!("(Table listing not yet implemented)");
    Ok(())
}

fn execute_sql(
    database: &mut Database,
    sql: &str,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let result = database.execute(sql)?;
    let duration = start.elapsed();

    println!("Query executed in {:.2}ms", duration.as_millis());
    Ok(result)
}

fn print_result(result: &QueryResult) {
    if result.rows.is_empty() {
        println!("(0 rows)");
        return;
    }

    // Print column headers
    if let Some(first_row) = result.rows.first() {
        for (i, _value) in first_row.values.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{:>10}", format!("col_{}", i));
        }
        println!();

        // Print separator
        for i in 0..first_row.values.len() {
            if i > 0 {
                print!("-+-");
            }
            print!("{:-<10}", "");
        }
        println!();
    }

    // Print data rows
    for row in &result.rows {
        for (i, value) in row.values.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{:>10}", format_value(value));
        }
        println!();
    }

    println!("({} rows)", result.rows.len());
}

fn format_value(value: &minidb::Value) -> String {
    match value {
        minidb::Value::Null => "NULL".to_string(),
        minidb::Value::Integer(i) => i.to_string(),
        minidb::Value::BigInt(i) => i.to_string(),
        minidb::Value::Float(f) => format!("{:.2}", f),
        minidb::Value::Double(f) => format!("{:.2}", f),
        minidb::Value::Varchar(s) => s.clone(),
        minidb::Value::Boolean(b) => b.to_string(),
        minidb::Value::Date(d) => d.to_string(),
        minidb::Value::Timestamp(ts) => ts.to_string(),
    }
}
