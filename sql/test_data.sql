USE neucourts;

-- test users
INSERT INTO users VALUES ('test1', 'test');
INSERT INTO users VALUES ('test2', 'test');
INSERT INTO users VALUES ('test3', 'test3');

-- test reservations
INSERT INTO reservations (username, start_time, end_time, court_id, party_id)
VALUES ('test1', NOW(), (SELECT DATE_ADD(NOW(), INTERVAL 15 MINUTE)), 1, NULL);

INSERT INTO reservations (username, start_time, end_time, court_id, party_id)
VALUES ('test2', NOW(), (SELECT DATE_ADD(NOW(), INTERVAL 15 MINUTE)), 2, NULL);

INSERT INTO reservations (username, start_time, end_time, court_id, party_id)
VALUES ('test2', NOW(), (SELECT DATE_ADD(NOW(), INTERVAL 15 MINUTE)), 3, NULL);
