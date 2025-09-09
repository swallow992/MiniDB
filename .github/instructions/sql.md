# SQL编译器模块指令

## 模块职责
该模块负责将SQL语句转换为可执行的查询计划，包含词法分析、语法分析、语义分析和执行计划生成四个阶段。

## 开发指导

### 词法分析器 (lexer.rs)
- **核心功能**: 将SQL文本转换为Token流
- **Token类型**: 关键字、标识符、数字、字符串、操作符、分隔符
- **错误处理**: 提供详细的词法错误位置信息
- **性能要求**: 支持流式处理，避免全文本加载

```rust
// 推荐的Token结构
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // SQL关键字
    Select,
    From,
    Where,
    // 标识符和字面量
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    // 操作符
    Plus,
    Minus,
    Equal,
    // 特殊Token
    EOF,
}

#[derive(Debug, Clone)]
pub struct TokenWithLocation {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}
```

### 语法分析器 (parser.rs)
- **解析策略**: 使用递归下降或解析器组合器
- **AST设计**: 类型安全的抽象语法树
- **支持语句**: SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE
- **错误恢复**: 实现错误恢复机制，继续解析后续语句

```rust
// 推荐的AST节点结构
#[derive(Debug, Clone)]
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
}

#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub select_list: Vec<SelectItem>,
    pub from_clause: Option<FromClause>,
    pub where_clause: Option<Expression>,
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order_by: Vec<OrderByItem>,
    pub limit: Option<u64>,
}
```

### 语义分析器 (analyzer.rs)
- **类型系统**: 实现SQL类型系统，支持类型推断和检查
- **作用域管理**: 处理表别名、列引用、子查询作用域
- **约束检查**: 验证外键约束、唯一性约束等
- **函数解析**: 支持内置函数和聚合函数的语义检查

```rust
// 推荐的符号表结构
pub struct SymbolTable {
    tables: HashMap<String, TableInfo>,
    current_scope: Scope,
}

pub struct TableInfo {
    pub columns: Vec<ColumnInfo>,
    pub primary_key: Option<String>,
    pub indexes: Vec<IndexInfo>,
}
```

### 执行计划生成器 (planner.rs)
- **计划树**: 生成关系代数形式的执行计划
- **优化规则**: 实现基本查询优化（谓词下推、连接重排序）
- **成本模型**: 简单的基于规则的成本估算
- **统计信息**: 集成表统计信息用于计划选择

```rust
// 推荐的执行计划节点
#[derive(Debug, Clone)]
pub enum PlanNode {
    TableScan { table: String, filter: Option<Expression> },
    IndexScan { table: String, index: String, range: Range },
    Join { left: Box<PlanNode>, right: Box<PlanNode>, condition: Expression },
    Projection { input: Box<PlanNode>, columns: Vec<String> },
    Sort { input: Box<PlanNode>, order: Vec<OrderByItem> },
}
```

## 代码生成约定

### 错误类型定义
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlError {
    #[error("词法错误: {message} at line {line}, column {column}")]
    LexError { message: String, line: usize, column: usize },
    
    #[error("语法错误: {message}")]
    ParseError { message: String },
    
    #[error("语义错误: {message}")]
    SemanticError { message: String },
    
    #[error("计划生成错误: {message}")]
    PlanError { message: String },
}
```

### 测试指导
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let sql = "SELECT id, name FROM users WHERE age > 18";
        let tokens = tokenize(sql).expect("词法分析失败");
        let stmt = parse_tokens(tokens).expect("语法分析失败");
        
        match stmt {
            Statement::Select(select) => {
                assert_eq!(select.select_list.len(), 2);
                assert!(select.where_clause.is_some());
            }
            _ => panic!("期望SELECT语句"),
        }
    }
}
```

## 性能考虑

### 内存管理
- 使用字符串内化（string interning）减少内存占用
- 实现AST节点的对象池以减少分配开销
- 考虑使用arena分配器管理AST生命周期

### 解析优化
- 实现增量解析支持IDE场景
- 使用预计算的关键字哈希表
- 考虑并行解析独立语句

## 调试支持

### 可视化工具
- 提供AST打印功能，便于调试
- 实现执行计划的图形化表示
- 支持解析过程的步进调试

### 诊断信息
- 详细的错误消息，包含修复建议
- 性能分析接口，测量各阶段耗时
- 内存使用统计

这些指导原则确保SQL编译器模块的代码质量、性能和可维护性。
