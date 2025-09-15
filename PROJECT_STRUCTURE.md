# MiniDB 项目结构（精简版）

## 📁 项目概述

MiniDB 是一个使用 Rust 语言构建的小型数据库系统，实现了SQL编译器、存储系统和查询执行引擎。经过优化清理，项目结构简洁清晰。

## 🏗️ 精简后的核心架构

```
MiniDB/
├── src/                          # 源代码目录
│   ├── lib.rs                    # 库根文件，定义公共接口
│   ├── main.rs                   # 主程序入口，交互式shell
│   ├── sql_compiler_demo.rs      # SQL编译器演示程序
│   ├── storage_demo.rs           # 存储系统演示程序  
│   ├── storage_system_test.rs    # 存储系统测试程序
│   ├── database_demo.rs          # 数据库系统演示程序
│   ├── sql/                      # SQL编译器模块
│   │   ├── mod.rs                # SQL模块定义
│   │   ├── lexer.rs              # 词法分析器
│   │   ├── lexer_tests.rs        # 词法分析器测试
│   │   ├── parser.rs             # 语法分析器
│   │   ├── analyzer.rs           # 语义分析器
│   │   └── planner.rs            # 执行计划生成器
│   ├── storage/                  # 存储系统模块
│   │   ├── mod.rs                # 存储模块定义
│   │   ├── page.rs               # 页面管理
│   │   ├── buffer.rs             # 缓冲池管理
│   │   ├── file.rs               # 文件系统接口
│   │   └── index.rs              # 索引系统
│   ├── engine/                   # 数据库引擎模块
│   │   ├── mod.rs                # 引擎模块定义
│   │   ├── database.rs           # 数据库核心
│   │   ├── executor.rs           # 执行器
│   │   ├── table.rs              # 表管理
│   │   ├── transaction.rs        # 事务管理
│   │   └── tests.rs              # 引擎测试
│   ├── types/                    # 类型定义模块
│   │   └── mod.rs                # 通用类型定义
│   └── utils/                    # 工具函数模块
│       ├── mod.rs                # 工具模块定义
│       ├── bitset.rs             # 位集合工具
│       ├── hash.rs               # 哈希工具
│       └── serialize.rs          # 序列化工具
├── .github/                      # GitHub配置
│   ├── AGENTS.md                 # 代理说明
│   ├── copilot-instructions.md   # Copilot指令
│   ├── instructions/             # 详细指令文件
│   └── prompts/                  # 提示词模板
├── target/                       # 编译输出目录
├── run_sql_compiler.ps1          # SQL编译器测试脚本
├── run_storage_test.ps1          # 存储系统测试脚本
├── sql_tests.sql                 # SQL测试用例
├── Cargo.toml                    # 项目配置
├── README.md                     # 项目说明
├── TESTING_GUIDE.md              # 测试指南
├── SQL_COMPILER_GUIDE.md         # SQL编译器指南
└── PROJECT_STRUCTURE.md          # 项目结构说明（本文件）
```

## 🎯 核心模块说明

### 1. SQL编译器模块 (`sql/`)
- **词法分析器**: 将SQL文本转换为token流
- **语法分析器**: 构建抽象语法树(AST) 
- **语义分析器**: 类型检查和语义验证
- **执行计划生成器**: 生成优化的执行计划

### 2. 存储系统模块 (`storage/`)
- **页面管理**: 4KB页面的分配和管理
- **缓冲池**: LRU缓存机制
- **文件系统**: 底层文件I/O操作
- **索引系统**: B+树索引实现

### 3. 数据库引擎模块 (`engine/`)
- **数据库核心**: 主要数据库接口
- **执行器**: SQL语句执行引擎
- **表管理**: 表的创建和管理
- **事务管理**: 基础事务支持

### 4. 演示程序
- **sql_compiler_demo.rs**: SQL编译器功能演示
- **storage_demo.rs**: 存储系统功能演示
- **storage_system_test.rs**: 存储系统性能测试
- **database_demo.rs**: 完整数据库系统演示

## 🧪 测试框架

### 核心测试脚本
1. **`run_sql_compiler.ps1`**: SQL编译器测试
   - 词法分析测试
   - 语法分析测试
   - 语义分析测试
   - 执行计划生成测试

2. **`run_storage_test.ps1`**: 存储系统测试
   - 页面分配测试
   - 缓存性能测试
   - 数据操作测试
   - 持久化测试

3. **`cargo run --bin database_demo`**: 数据库系统测试
   - 完整SQL操作流程测试
   - 数据持久性验证
   - 系统集成测试

## 📚 文档结构

- **README.md**: 项目简介和快速开始
- **TESTING_GUIDE.md**: 详细测试指南和使用说明
- **SQL_COMPILER_GUIDE.md**: SQL编译器详细说明
- **PROJECT_STRUCTURE.md**: 项目结构说明（本文件）

## 🔧 构建和运行

```bash
# 构建项目
cargo build

# 运行主程序
cargo run

# 运行特定演示程序
cargo run --bin sql_compiler_demo
cargo run --bin storage_demo
cargo run --bin database_demo

# 运行测试
./run_sql_compiler.ps1
./run_storage_test.ps1
```

## 🎓 学习价值

### 数据库核心概念
- **SQL解析**: 词法分析 → 语法分析 → 语义分析
- **存储管理**: 页式存储 → 缓冲池 → 文件系统
- **查询执行**: 执行计划 → 操作符 → 结果生成
- **事务处理**: ACID属性 → 并发控制 → 恢复机制

### Rust编程实践
- **模块化设计**: 清晰的模块边界和接口
- **错误处理**: Result类型和错误传播
- **内存安全**: 所有权系统和借用检查
- **性能优化**: 零成本抽象和编译时优化

## 📈 扩展方向

- **查询优化**: 基于成本的优化器
- **索引系统**: B+树、哈希索引
- **并发控制**: 锁机制、MVCC
- **网络协议**: 客户端-服务器架构
- **SQL支持**: 更多SQL标准特性

---

这个精简版的MiniDB项目保持了核心功能的完整性，同时消除了冗余和过时的代码，为学习数据库系统实现提供了清晰的示例。