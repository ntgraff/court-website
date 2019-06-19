USE neucourts;

-- Table to hold court information
CREATE TABLE IF NOT EXISTS courts (
    court_id    INT         NOT NULL AUTO_INCREMENT,
    name        VARCHAR(30) NOT NULL,
    description VARCHAR(255) NULL,
    PRIMARY KEY (court_id)
);

CREATE TABLE IF NOT EXISTS court_types (
    type_name VARCHAR(100) NOT NULL,
    type_desc VARCHAR(255) NULL     DEFAULT(type_name),
    PRIMARY KEY (type_name)
);

CREATE TABLE IF NOT EXISTS type_registrar (
    court_id  INT          NOT NULL,
    type_name VARCHAR(100) NOT NULL,
    PRIMARY KEY (court_id, type_name),
    FOREIGN KEY (court_id) REFERENCES courts(court_id),
    FOREIGN KEY (type_name) REFERENCES court_types(type_name)
);

-- Table to hold users information
CREATE TABLE IF NOT EXISTS users (
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    PRIMARY KEY (username)
);

-- Table to hold parties
CREATE TABLE IF NOT EXISTS parties (
    party_id INT          NOT NULL  AUTO_INCREMENT UNIQUE,
    capacity INT          NOT NULL,
    PRIMARY KEY (party_id),
    CONSTRAINT valid_capacity CHECK(capacity > 0)
);

CREATE TABLE IF NOT EXISTS party_registrar (
    party_id INT          NOT NULL,
    user     VARCHAR(255) NOT NULL,
    PRIMARY KEY (party_id, user),
    FOREIGN KEY (party_id) REFERENCES parties(party_id),
    FOREIGN KEY (user)     REFERENCES users(username)
);

-- Table to hold intended use of a court by an independent party
CREATE TABLE IF NOT EXISTS reservations (
    reservation_id INT          NOT NULL AUTO_INCREMENT,
    username       VARCHAR(255) NOT NULL,
    start_time     DATETIME     NOT NULL,
    end_time       DATETIME     NOT NULL,
    court_id       INT          NOT NULL,
    party_id       INT,
    PRIMARY KEY (reservation_id),
    FOREIGN KEY (username) REFERENCES users(username),
    FOREIGN KEY (court_id) REFERENCES courts(court_id),
    FOREIGN KEY (party_id) REFERENCES parties(party_id),
    CONSTRAINT valid_start_end CHECK(start_time <= end_time)
);
