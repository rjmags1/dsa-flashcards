CREATE TABLE question (
    id INTEGER PRIMARY KEY NOT NULL,
    question_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    title_slug TEXT NOT NULL,
    prompt TEXT NOT NULL,
    difficulty TEXT NOT NULL,
    fetched BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE question_topic (
    id INTEGER PRIMARY KEY NOT NULL,
    question_number INTEGER NOT NULL,
    topic TEXT NOT NULL
);