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
	FROM intended_use
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
-- A function that checks if a login was successful (username, password) -> boolean
-- A function that checks if a user can register (username) -> boolean
-- A procedure to register a user (username, password) => modify users table
-- One of those trigger things that removes a reservation when it is over maybe?
--   not sure if we need this one
-- Add cascade stuff to the tables
-- Maybe add more constraints to the tables? not sure.
-- We will need a few more, but I don't know yet
