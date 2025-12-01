CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id INTEGER NOT NULL,
    datetime DATETIME NOT NULL,
    session_number INTEGER NOT NULL,
    height REAL,
    weight REAL,
    anterior TEXT,
    posterior TEXT,
    right_lateral TEXT,
    left_lateral TEXT,
    notes TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE
);