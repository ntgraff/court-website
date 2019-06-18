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

-- Table to hold intended use of a court by an independent party
CREATE TABLE IF NOT EXISTS intended_use (
    intended_use_id INT          NOT NULL AUTO_INCREMENT,
    username        VARCHAR(255) NOT NULL,
    start_time      DATETIME     NOT NULL,
    end_time        DATETIME     NOT NULL,
    court_id        INT          NOT NULL,
    PRIMARY KEY (intended_use_id),
    FOREIGN KEY (username) REFERENCES users(username),
    FOREIGN KEY (court_id) REFERENCES courts(court_id)
);

