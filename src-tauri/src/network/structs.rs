use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseTopic {
    pub name: String,
    pub id: String,
    pub slug: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseQuestion {
    pub difficulty: String,
    pub questionId: String,
    pub title: String,
    pub titleSlug: String,
    pub topicTags: Vec<ResponseTopic>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionList {
    pub questions: Vec<ResponseQuestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemsetQuestionList {
    pub problemsetQuestionList: QuestionList,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionListResponse {
    pub data: ProblemsetQuestionList,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionPromptContent {
    pub content: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionPrompt {
    pub question: QuestionPromptContent
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptResponse {
    pub data: QuestionPrompt
}
