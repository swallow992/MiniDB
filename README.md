# MiniDB

一个使用Rust语言构建的小型数据库系统，用于学习和理解数据库系统的核心概念。

## 项目概述

MiniDB旨在分步构建一个完整的数据库系统，包含以下核心组件：

### 1. SQL编译器
- **词法分析器**: 将SQL文本转换为Token流
- **语法分析器**: 构建抽象语法树(AST)
- **语义分析器**: 类型检查和作用域解析
- **查询计划器**: 生成和优化执行计划

### 2. 存储系统
- **页式存储**: 4KB固定大小页面管理
- **缓冲池**: LRU替换策略的内存缓存
- **文件管理**: 数据文件的创建、读写和管理
- **索引结构**: B+树索引实现

### 3. 查询引擎
- **执行器**: 火山模型的查询执行框架
- **算子实现**: 扫描、连接、聚合、排序等
- **事务处理**: ACID事务保证
- **并发控制**: 多版本并发控制(MVCC)

## 技术特点

- ✅ **类型安全**: 利用Rust的类型系统确保内存安全
- ✅ **高性能**: 零成本抽象和SIMD优化
- ✅ **并发友好**: 基于Rust的并发原语实现
- ✅ **可扩展**: 模块化设计便于功能扩展
- ✅ **易调试**: 详细的错误信息和日志系统

## 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 安装和运行

```bash
# 克隆项目
git clone https://github.com/swallow992/MiniDB.git
cd MiniDB

# 构建项目
cargo build --release

# 运行测试
cargo test

# 运行数据库
cargo run --bin minidb
```

## GitHub Copilot 配置

本项目已配置完整的GitHub Copilot工作流，包括：

- `.github/copilot-instructions.md` - 主要仓库指令
- `.github/instructions/` - 路径特定指令（sql.md, storage.md, engine.md）
- `.github/prompts/` - 可复用提示模板
- `.github/AGENTS.md` - AI代理指令文档

这些配置文件将帮助GitHub Copilot为您生成符合项目标准的高质量Rust代码。

## 贡献指南

欢迎为MiniDB项目做出贡献！请查看我们的贡献指南了解详细信息。

## 许可证

本项目使用MIT许可证 - 查看[LICENSE](LICENSE)文件了解详情。