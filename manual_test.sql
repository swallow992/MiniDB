CREATE TABLE demo (id INT, name TEXT, value INT);
INSERT INTO demo VALUES (1, 'Test1', 100);
INSERT INTO demo VALUES (2, 'Test2', 200);
SELECT * FROM demo;
SELECT name FROM demo WHERE value > 150;
UPDATE demo SET value = 300 WHERE id = 1;
SELECT * FROM demo;
DELETE FROM demo WHERE id = 2;
SELECT * FROM demo;