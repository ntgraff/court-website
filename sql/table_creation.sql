-- Table to hold court information
CREATE TABLE IF NOT EXISTS courts (
    court_id           INT         NOT NULL AUTO_INCREMENT,
    name               VARCHAR(30) NOT NULL,
    court_type         VARCHAR(100) NOT NULL,
    PRIMARY KEY (court_id)
);

-- Table to hold users information
CREATE TABLE IF NOT EXISTS users (
    username VARCHAR(45) NOT NULL UNIQUE,
    password VARCHAR(30) NOT NULL,
    PRIMARY KEY (username)
);

-- Table to hold all party requests
CREATE TABLE IF NOT EXISTS open_party (
    party_id             INT         NOT NULL AUTO_INCREMENT,
    host                 VARCHAR(45) NOT NULL,
    current_party_size   INT         NOT NULL,
    remaining_open_slots INT         NOT NULL,
    court                INT         NOT NULL,
    intended_play_time   DATETIME    NOT NULL,
    PRIMARY KEY (party_id),
    FOREIGN KEY (host) REFERENCES users(username),
    FOREIGN KEY (court) REFERENCES courts(court_id)
);

-- Table to hold scheduled events affiliated with the university
CREATE TABLE IF NOT EXISTS scheduled_event (
    scheduled_event_id   INT          NOT NULL AUTO_INCREMENT,
    event_name           VARCHAR(255) NOT NULL,
    scheduled_start_time DATETIME     NOT NULL,
    scheduled_end_time   DATETIME     NOT NULL,
    court                INT          NOT NULL,
    PRIMARY KEY (scheduled_event_id),
    FOREIGN KEY (court) REFERENCES courts(court_id)
);

-- Table to hold intended use of a court by an independent party
CREATE TABLE IF NOT EXISTS intended_use (
    intended_use_id   INT          NOT NULL AUTO_INCREMENT,
    username          VARCHAR(255) NOT NULL,
    intended_end_time DATETIME     NOT NULL,
    court             INT          NOT NULL,
    PRIMARY KEY (intended_use_id),
    FOREIGN KEY (username) REFERENCES users(username),
    FOREIGN KEY (court) REFERENCES courts(court_id)
);

CREATE TABLE IF NOT EXISTS expected_occupancy (
    court_id       INT       NOT NULL,
    occupied       INT       NOT NULL,
    occupied_until DATETIME  NOT NULL,
    PRIMARY KEY (court_id),
    FOREIGN KEY (court_id) REFERENCES courts(court_id)
);
