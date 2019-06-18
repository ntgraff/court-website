-- reserves court from now until the specifed time, under the specifed user.
DROP PROCEDURE IF EXISTS add_reservation_until_if_free;

DELIMITER ;;
CREATE PROCEDURE add_reservation_until_if_free( cid INT, until DATETIME, username VARCHAR(45) )
BEGIN
	DECLARE is_open INT;
	SET is_open = -1;
	SELECT court_id INTO is_open
	FROM intended_use
	WHERE court_id = cid AND (start_time > until OR end_time < NOW());
	IF is_open = -1 THEN
		INSERT INTO intended_use (username, start_time, end_time, court_id) VALUES (username, NOW(), until, cid);
	END IF;
END ;;
DELIMITER ;

-- checks if a court is occupied
DROP FUNCTION IF EXISTS is_occupied;

DELIMITER ;;
CREATE FUNCTION is_occupied( cid INT )
RETURNS BOOLEAN
BEGIN
	DECLARE used_count INT;
	SELECT COUNT(intended_use_id) INTO used_count
	FROM intended_use
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
		intended_use_id,
		username,
		CONVERT(start_time, VARCHAR(50)),
		CONVERT(end_time, VARCHAR(50)),
		court_id
	FROM intended_use
	WHERE court_id = cid AND end_time > NOW()
	ORDER BY start_time DESC;
END ;;
DELIMITER ;
