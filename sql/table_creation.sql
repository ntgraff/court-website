-- Table to hold court information
CREATE TABLE IF NOT EXISTS courts (
	court_id INT NOT NULL AUTO_INCREMENT,
	name VARCHAR(30) NOT NULL,
	occupied BIT(1) NOT NULL,
    expected_occupancy DATETIME,
	court_type VARCHAR(45) NOT NULL,
	PRIMARY KEY (court_id)
);

-- Table to hold user information
CREATE TABLE IF NOT EXISTS user (
	username VARCHAR(45) NOT NULL UNIQUE,
	password VARCHAR(30) NOT NULL,
	PRIMARY KEY (username)
);

-- Table to hold all party requests
CREATE TABLE IF NOT EXISTS open_party (
	party_id INT NOT NULL AUTO_INCREMENT,
    host VARCHAR(45) NOT NULL,
    current_party_size INT NOT NULL,
    remaining_open_slots INT NOT NULL,
    court INT NOT NULL,
	intended_play_time DATETIME NOT NULL,
	PRIMARY KEY (party_id),
    FOREIGN KEY (host) REFERENCES user(username),
    FOREIGN KEY (court) REFERENCES courts(court_id)
);

-- Table to hold scheduled events affiliated with the university
CREATE TABLE IF NOT EXISTS scheduled_event (
	scheduled_event_id INT NOT NULL AUTO_INCREMENT,
	event_name VARCHAR(255) NOT NULL,
    scheduled_start_time DATETIME NOT NULL,
    scheduled_end_time DATETIME NOT NULL,
    court INT NOT NULL,
	PRIMARY KEY (user_id)
);
