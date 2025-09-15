# MiniDB 交互模式功能需求分析

## 当前交互模式限制

### 缺失的功能：
- CREATE TABLE 命令
- INSERT 命令  
- 无法手动构造测试数据

### 现有功能：
- show tables（显示表列表）
- show <table>（显示表内容）
- select <table>（执行查询）
- delete <table>（删除记录）
- test（运行完整测试）

## 需求分析

### 学术/教学需求 - **建议添加**
如果用于：
- 课程演示
- 学生实验
- 交互式探索数据库功能
- 手动验证特定场景

**推荐添加功能：**
```
create table <name>     - 创建预定义模板表
insert <table> <data>   - 插入简化数据
stats                   - 显示存储统计
cache                   - 显示缓存状态
```

### 工程/测试需求 - **脚本足够**
如果用于：
- 自动化测试
- 性能验证
- 系统集成测试
- CI/CD流程

**当前脚本已满足：**
- storage_demo.exe（存储系统测试）
- database_demo.exe test（数据库完整测试）
- SQL编译器独立测试

## 建议方案

### 方案1：添加简化的交互命令（推荐）
优点：
- 支持手动探索和验证
- 教学演示友好
- 可以验证特定边界情况

缺点：
- 需要额外开发工作
- 命令解析复杂度增加

### 方案2：保持现状，只用脚本测试
优点：
- 开发工作量小
- 自动化测试已完整
- 减少维护成本

缺点：
- 无法手动验证特定场景
- 交互体验受限

## 推荐实现（如果选择方案1）

### 简化的CREATE TABLE命令
```
create users       - 创建用户表模板
create orders      - 创建订单表模板  
create products    - 创建产品表模板
create test <name> - 创建简单测试表
```

### 简化的INSERT命令
```
insert users alice 25           - 插入用户数据
insert orders 101 1 99.99       - 插入订单数据
insert test <table> <value>     - 插入测试数据
```

### 存储统计命令
```
stats       - 显示存储系统统计
cache       - 显示缓存状态
pages       - 显示页面分配情况
```