CREATE TABLE USERS (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT NOT NULL UNIQUE
);

CREATE TABLE PASSWORDS (
  userid TEXT PRIMARY KEY,
  salt TEXT NOT NULL,
  hash TEXT NOT NULL,
  FOREIGN KEY (userid) REFERENCES USERS(id)
);

