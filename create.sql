CREATE TABLE users (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    username VARCHAR(30) NOT NULL UNIQUE,
    email VARCHAR(254) NOT NULL UNIQUE,
    password_hash VARCHAR(128) NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    discord_id BIGINT UNSIGNED,

    upgraded BOOL NOT NULL DEFAULT 0,
    upgraded_at TIMESTAMP,

    PRIMARY KEY (id)
);

CREATE TABLE friends (
    user_id1 INT UNSIGNED NOT NULL,
    user_id2 INT UNSIGNED NOT NULL,

    FOREIGN KEY (user_id1) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id2) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE fund_sources (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    user_id INT UNSIGNED NOT NULL,

    name VARCHAR(100) NOT NULL,

    default_currency VARCHAR(3) NOT NULL DEFAULT 'GBP',

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

    PRIMARY KEY (id)
);

CREATE TABLE budgets (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    fund_source_id INT UNSIGNED NOT NULL,

    name VARCHAR(100) NOT NULL,

    capacity INT UNSIGNED NOT NULL DEFAULT 100,

    last_reset_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (fund_source_id) REFERENCES fund_sources(id) ON DELETE CASCADE,

    PRIMARY KEY (id)
);

CREATE TABLE transactions (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    fund_source_id INT UNSIGNED NOT NULL,
    budget_id INT UNSIGNED,

    volume INT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'GBP',

    notes TEXT,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (fund_source_id) REFERENCES fund_sources(id) ON DELETE CASCADE,
    FOREIGN KEY (budget_id) REFERENCES budgets(id) ON DELETE SET NULL,

    PRIMARY KEY (id)
);

CREATE TABLE ious (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,

    owing_user INT UNSIGNED NOT NULL,
    owed_user INT UNSIGNED NOT NULL,

    owed_amount INT UNSIGNED NOT NULL DEFAULT 1,
    notes TEXT,
    currency VARCHAR(3) NOT NULL DEFAULT 'GBP',

    accepted BOOL,

    FOREIGN KEY (owing_user) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (owed_user) REFERENCES users(id) ON DELETE CASCADE,

    PRIMARY KEY (id)
);

CREATE TABLE receipts (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    user_id INT UNSIGNED NOT NULL,

    image MEDIUMBLOB NOT NULL,
    notes TEXT,

    uploaded_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

    PRIMARY KEY (id)
);
