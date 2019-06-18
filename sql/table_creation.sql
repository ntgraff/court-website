-- Table to hold court information
CREATE TABLE IF NOT EXISTS courts (
    court_id           INT         NOT NULL AUTO_INCREMENT,
    name               VARCHAR(30) NOT NULL,
    court_type         VARCHAR(100) NOT NULL,
    PRIMARY KEY (court_id)
);

-- Table to hold users information
CREATE TABLE IF NOT EXISTS users (
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    PRIMARY KEY (username)
);

-- Table to hold intended use of a court by an independent party
CREATE TABLE IF NOT EXISTS reservations (
    reservation_id INT          NOT NULL AUTO_INCREMENT,
    username       VARCHAR(255) NOT NULL,
    start_time     DATETIME     NOT NULL DEFAULT NOW(),
    end_time       DATETIME     NOT NULL,
    court_id       INT          NOT NULL,
    party_id       INT,
    PRIMARY KEY (reservation_id),
    FOREIGN KEY (username) REFERENCES users(username),
    FOREIGN KEY (court_id) REFERENCES courts(court_id),
    FOREIGN KEY (party_id) REFERENCES parties(party_id),
    CONSTRAINT valid_start_end CHECK(start_time <= end_time)
);

-- Table to hold parties
CREATE TABLE IF NOT EXISTS parties {
    party_id INT NOT NULL AUTO_INCREMENT,
    username VARCHAR(255) NOT NULL,
    capacity INT NOT NULL,
    current INT NOT NULL DEFAULT 0,
    PRIMARY KEY (party_id),
    FOREIGN KEY (username) REFERENCES users(username),
    CONSTRAINT valid_current CHECK(current <= capacity)
}
