DROP TABLE IF EXISTS question;
DROP TABLE IF EXISTS question_topic;

CREATE TABLE source (
    sid INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE question (
    qid INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    title_slug TEXT,
    prompt TEXT DEFAULT "",
    difficulty TEXT CHECK( difficulty in ("EASY", "MEDIUM", "HARD") ),
    source INTEGER,
    source_qid INTEGER,
    FOREIGN KEY(source) REFERENCES source(sid)
);

CREATE TABLE topic (
    tid INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE question_topic (
    relid INTEGER PRIMARY KEY NOT NULL,
    qid INTEGER NOT NULL,
    tid INTEGER NOT NULL,
    FOREIGN KEY(qid) REFERENCES new_question(qid),
    FOREIGN KEY(tid) REFERENCES new_question_topic(tid)
);