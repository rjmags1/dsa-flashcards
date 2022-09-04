use super::schema::{question, question_topic, topic};

#[derive(Queryable)]
pub struct Question {
    pub qid: i32,
    pub title: String,
    pub title_slug: Option<String>,
    pub prompt: Option<String>,
    pub difficulty: Option<String>,
    pub source: Option<i32>,
    pub source_qid: Option<i32>,
}

#[derive(Queryable, Clone)]
pub struct Topic {
    pub tid: i32,
    pub name: String,
}

#[derive(Queryable)]
pub struct QuestionTopic {
    pub relid: i32,
    pub qid: i32,
    pub tid: i32,
}

#[derive(Queryable)]
pub struct Star {
    pub relid: i32,
    pub qid: i32,
    pub uid: i32,
}

#[derive(Queryable)]
pub struct User {
    pub uid: i32,
    pub name: String,
    pub hide_diff: bool,
    pub hide_cat: bool,
    pub hide_solved: bool,
    pub dark_mode: bool,
}

#[derive(Queryable)]
pub struct Solution {
    pub sid: i32,
    pub uid: i32,
    pub qid: i32,
    pub notes: String,
}



#[derive(Insertable)]
#[table_name="question"]
pub struct NewQuestion {
    pub title: String,
    pub title_slug: String,
    pub prompt: String,
    pub difficulty: String,
    pub source: i32,
    pub source_qid: i32,
}

#[derive(Insertable)]
#[table_name="topic"]
pub struct NewTopic {
    pub name: String
}

#[derive(Insertable)]
#[table_name="question_topic"]
pub struct NewQuestionTopic {
    pub qid: i32,
    pub tid: i32,
}
