use super::schema::{question, question_topic};

#[derive(Queryable)]
#[derive(QueryableByName)]
#[table_name="question"]
pub struct Question {
    pub id: i32,
    pub question_number: i32,
    pub title: String,
    pub title_slug: String,
    pub prompt: String,
    pub difficulty: String,
    pub fetched: bool,
}

#[derive(Queryable)]
pub struct QuestionTopic {
    pub id: i32,
    pub question_number: i32,
    pub topic: String,
}

#[derive(Insertable)]
#[table_name="question"]
pub struct NewQuestion {
    pub question_number: i32,
    pub title: String,
    pub title_slug: String,
    pub prompt: String,
    pub difficulty: String,
    pub fetched: bool
}

#[derive(Insertable)]
#[table_name="question_topic"]
pub struct NewQuestionTopic {
    pub question_number: i32,
    pub topic: String,
}