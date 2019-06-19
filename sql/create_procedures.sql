-- reserves court from start until end, under the specifed user.
DROP PROCEDURE IF EXISTS add_reservation;

DELIMITER ;;
CREATE PROCEDURE add_reservation( cid INT, start DATETIME, end DATETIME, username VARCHAR(255) )
BEGIN
	INSERT INTO reservations (username, start_time, end_time, court_id) VALUES (username, start, end, cid);
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS add_reservation_with_party;

DELIMITER ;;
CREATE PROCEDURE add_reservation_with_party( cid INT, start DATETIME, end DATETIME, username VARCHAR(255), pca INT )
BEGIN
	DECLARE pid INT;
	INSERT INTO parties (capacity) VALUES (pca);
	SELECT LAST_INSERT_ID() INTO pid;
	INSERT INTO party_registrar (party_id, user) VALUES (pid, username);
	INSERT INTO reservations (username, start_time, end_time, court_id, party_id)
	VALUES (username, start, end, cid, pid);
END ;;
DELIMITER ;

-- checks if there is a reservation between the times on that court
DROP FUNCTION IF EXISTS can_reserve_between;
DELIMITER ;;
CREATE FUNCTION can_reserve_between( cid INT, start DATETIME, endt DATETIME )
RETURNS BOOLEAN
BEGIN
	DECLARE resc_at_time INT;
	SELECT COUNT(reservation_id) INTO resc_at_time
	FROM reservations
	WHERE court_id = cid AND (start_time BETWEEN start AND endt OR end_time BETWEEN start AND endt);
	RETURN resc_at_time = 0;
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
	WHERE court_id = cid AND end_time >= NOW()
	ORDER BY start_time ASC;
END ;;
DELIMITER ;

DROP FUNCTION IF EXISTS party_current_count;

DELIMITER ;;
CREATE FUNCTION party_current_count( pid INT ) RETURNS INT
BEGIN
	DECLARE curc INT;
	SELECT COUNT(party_id) INTO curc FROM party_registrar WHERE party_id = pid;
	RETURN curc;
END ;;

DELIMITER ;

DROP PROCEDURE IF EXISTS party_details;

DELIMITER ;;
CREATE PROCEDURE party_details( pid INT )
BEGIN
	SELECT party_id, capacity, party_current_count(party_id) AS current FROM parties WHERE party_id = pid;
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS can_join_party;

DELIMITER ;;
CREATE FUNCTION can_join_party( un VARCHAR(255), pid INT ) RETURNS BOOLEAN
BEGIN
	DECLARE curc INT;
	DECLARE cap  INT;
	SELECT party_current_count(pid) INTO curc;
	SELECT capacity INTO cap FROM parties WHERE party_id = pid;
	RETURN (un NOT IN (SELECT user FROM party_registrar WHERE party_id = pid)) AND (curc < cap);
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS try_join_party;

DELIMITER ;;
CREATE PROCEDURE try_join_party( un VARCHAR(255), pid INT )
BEGIN
	DECLARE can_join BOOLEAN;
	SELECT can_join_party(un, pid) INTO can_join;
	IF can_join THEN
		INSERT INTO party_registrar (party_id, user) VALUES (pid, un);
		SELECT TRUE;
	ELSE
		SELECT FALSE;
	END IF;
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS reservation_available_party;

DELIMITER ;;
CREATE PROCEDURE reservation_available_party( rid INT )
BEGIN
	DECLARE pid INT;
	SET pid = NULL;
	SELECT party_id INTO pid
	FROM reservations
	WHERE reservation_id = rid;

	CALL party_details(pid);
END ;;
DELIMITER ;

-- A function that checks if a login was successful (username, password) -> TINYINT(1 IS TRUE, 0 IS FALSE)
DROP FUNCTION IF EXISTS successful_login;

DELIMITER ;;

CREATE FUNCTION successful_login(un varchar(255), pw varchar(255)) RETURNS BOOLEAN
BEGIN
	DECLARE pass VARCHAR(255);
	SELECT password INTO pass
	FROM users
	WHERE username = un;
	RETURN pass = pw;
END ;;
DELIMITER ;

-- A function that checks if a user can register (username) -> TINYINT(1 IS TRUE, 0 IS FALSE)
DROP FUNCTION IF EXISTS available_username;

DELIMITER ;;
CREATE FUNCTION available_username(un varchar(255)) RETURNS BOOLEAN
BEGIN
	DECLARE users_with_name INT;
	SELECT count(username) INTO users_with_name
	FROM users
	WHERE username = un;
	RETURN users_with_name = 0;
END ;;
DELIMITER ;

-- A procedure to register a user (username, password) => modify users table
DROP PROCEDURE IF EXISTS  add_user;

DELIMITER ;;
CREATE PROCEDURE add_user(us VARCHAR(255), pw VARCHAR(255))
BEGIN
	INSERT INTO users(username, password) VALUES (us, pw);
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS try_register_user;

DELIMITER ;;
CREATE PROCEDURE try_register_user( un VARCHAR(255), pw1 VARCHAR(255), pw2 VARCHAR(255) )
BEGIN
	DECLARE un_available BOOLEAN;
	SELECT available_username(un) INTO un_available;
	IF un_available AND pw1 = pw2 THEN
		CALL add_user(un, pw1);
		SELECT TRUE;
	ELSE
		SELECT FALSE;
	END IF;
END ;;
DELIMITER ;

DROP PROCEDURE IF EXISTS court_types;

DELIMITER ;;
CREATE PROCEDURE court_types(cid INT)
BEGIN
	SELECT t.type_name, t.type_desc
	FROM type_registrar r
	JOIN court_types t ON r.type_name = t.type_name
	WHERE r.court_id = cid;
END ;;
DELIMITER ;

-- Add cascade stuff to the tables
-- Maybe add more constraints to the tables? not sure.
-- We will need a few more, but I don't know yet
