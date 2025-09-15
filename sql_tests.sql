CREATE TABLE student (id INT, name VARCHAR(50), score DECIMAL(5,2))
INSERT INTO student (id, name, score) VALUES (1, 'Alice', 95.5)
INSERT INTO student VALUES (2, 'Bob', 87.0)
SELECT * FROM student
SELECT name, score FROM student WHERE score > 90
CREATE TABLE teacher (id INT, name VARCHAR(100), subject VARCHAR(50))
INSERT INTO teacher VALUES (1, 'John Smith', 'Mathematics')
INSERT INTO teacher VALUES (2, 'Mary Johnson', 'Physics')
SELECT * FROM teacher
SELECT name FROM teacher WHERE id = 1