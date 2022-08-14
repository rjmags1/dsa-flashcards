CREATE TABLE question (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    prompt TEXT NOT NULL,
    difficulty TEXT NOT NULL CHECK(difficulty = "E" OR difficulty = "M" OR difficulty = "H")
);