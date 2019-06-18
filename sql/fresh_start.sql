DROP DATABASE IF EXISTS neucourts;

CREATE DATABASE neucourts;
USE neucourts;

source sql/table_creation.sql;
source sql/create_procedures.sql;

SET FOREIGN_KEY_CHECKS=0;
TRUNCATE courts;
SET FOREIGN_KEY_CHECKS=1;

LOAD DATA LOCAL INFILE 'sql/courts.csv'
INTO TABLE courts FIELDS TERMINATED BY ',' OPTIONALLY ENCLOSED BY '"'
LINES TERMINATED BY '\n'
IGNORE 1 LINES
(name, court_type);

