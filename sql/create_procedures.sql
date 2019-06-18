-- reserves court from start until end, under the specifed user.
DROP PROCEDURE IF EXISTS add_reservation;

DELIMITER ;;
CREATE PROCEDURE add_reservation( cid INT, start DATETIME,  end DATETIME, username VARCHAR(45) )
BEGIN
	INSERT INTO reservations (username, start_time, end_time, court_id) VALUES (username, start, end, cid);
END ;;
DELIMITER ;

-- checks if a court is occupied
DROP FUNCTION IF EXISTS is_occupied;

DELIMITER ;;
CREATE FUNCTION is_occupied( cid INT )
RETURNS BOOLEAN
BEGIN
	DECLARE used_count INT;
	SELECT COUNT(reservation_id) INTO used_count
	FROM reservations
	WHERE court_id = cid AND start_time < NOW() AND end_time > NOW();
	RETURN used_count <> 0;
END ;;
DELIMITER ;

-- What time is the court next free
DROP PROCEDURE IF EXISTS court_reservations;

DELIMITER ;;
CREATE PROCEDURE court_reservations( cid INT )
BEGIN
	SELECT
		reservation_id,
		username,
		CONVERT(start_time, VARCHAR(50)),
		CONVERT(end_time, VARCHAR(50)),
		court_id,
		party_id
	FROM reservations
	WHERE court_id = cid AND end_time > NOW()
	ORDER BY start_time DESC;
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS  reservation_available_party;

DELIMITER ;;
CREATE PROCEDURE reservation_available_party( rid INT )
BEGIN
	DECLARE pid INT;
	SET pid = NULL;
	SELECT party_id INTO pid
	FROM reservations
	WHERE reservation_id = rid;
	SELECT party_id, capacity, current
	FROM parties
	WHERE party_id = pid;
END ;;


-- A function that checks if a login was successful (username, password) -> TINYINT(1 IS TRUE, 0 IS FALSE)
drop function if exists succesful_login;

DELIMITER //

CREATE FUNCTION succesful_login(un varchar(255), pw varchar(255)) RETURNS TINYINT(1) DETERMINISTIC
BEGIN
	DECLARE num int;
    SELECT count(*) into num FROM users WHERE username = un and password = pw;
    return encounter;
END //

DELIMITER ;


-- A function that checks if a user can register (username) -> TINYINT(1 IS TRUE, 0 IS FALSE)
drop function if exists available_username;

DELIMITER //
CREATE FUNCTION available_username(un varchar(255)) RETURNS TINYINT(1) DETERMINISTIC
BEGIN
	DECLARE num int;
    SELECT count(*) into num FROM users WHERE username = un;
    if num = 1 then 
		return 0; 
    END IF;
    return 1;
END //

DELIMITER ;

-- A procedure to register a user (username, password) => modify users table
DROP PROCEDURE IF EXISTS  add_user;

DELIMITER ;;
CREATE PROCEDURE add_user(us VARCHAR(255), pw VARCHAR(255))
BEGIN
	INSERT INTO users(username, password) VALUES (us, pw);
END ;;


-- One of those trigger things that removes a reservation when it is over maybe?
-- Maybe instead a function that deletes reservations for a specific time period
-- This way it can keep old records in case someone wants to check something
-- and easily delete old records when needed
-- idk
--   not sure if we need this one
-- Add cascade stuff to the tables
-- Maybe add more constraints to the tables? not sure.
-- We will need a few more, but I don't know yet
