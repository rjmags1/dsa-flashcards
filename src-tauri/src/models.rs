#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub title: String,
    pub prompt: String,
    pub difficulty: String,
}