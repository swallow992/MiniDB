-- 创建员工表
CREATE TABLE employees (
    id INT,
    name TEXT,
    department TEXT,
    salary INT,
    hire_date TEXT
);

-- 插入测试数据
INSERT INTO employees VALUES (1, 'Alice Johnson', 'Engineering', 75000, '2023-01-15');
INSERT INTO employees VALUES (2, 'Bob Smith', 'Marketing', 65000, '2023-02-20');
INSERT INTO employees VALUES (3, 'Carol Davis', 'Engineering', 80000, '2023-03-10');
INSERT INTO employees VALUES (4, 'David Wilson', 'Sales', 55000, '2023-04-05');
INSERT INTO employees VALUES (5, 'Eva Brown', 'HR', 60000, '2023-05-12');

-- 基本查询测试
SELECT * FROM employees;
SELECT COUNT(*) FROM employees;
SELECT name, salary FROM employees;

-- 条件查询测试
SELECT * FROM employees WHERE department = 'Engineering';
SELECT * FROM employees WHERE salary > 70000;
SELECT name FROM employees WHERE hire_date LIKE '2023-01%';

-- 更新操作测试
UPDATE employees SET salary = 85000 WHERE name = 'Alice Johnson';
SELECT * FROM employees WHERE name = 'Alice Johnson';

-- 删除操作测试
DELETE FROM employees WHERE department = 'Sales';
SELECT * FROM employees;

-- 验证最终状态
SELECT COUNT(*) FROM employees;
