use serde_json::json;
use crate::network::structs::{QuestionList, QuestionListResponse, PromptResponse};
use crate::network::lc_graphql::*;

pub async fn fetch_all_questions() -> 
Result<QuestionList, Box<dyn std::error::Error>> {

    let req_body = json!({
        "query": Q_LIST_QUERY,
        "variables": {
            "categorySlug": "",
            "limit": LIST_LEN_LIMIT,
            "filters": {},
        }
    });
    let res = reqwest::Client::new()
        .post(LC_GRAPHQL_ENDPOINT)
        .json(&req_body)
        .send()
        .await?;
    let parsed = res.json::<QuestionListResponse>().await?;

    Ok(parsed.data.problemsetQuestionList)

}

pub async fn fetch_prompt(
    title_slug: &String
) -> Result<String, Box<dyn std::error::Error>> {

    let query_string = Q_PROMPT_QUERY;
    let req_body = json!({
        "query": query_string,
        "variables": {
            "titleSlug": title_slug,
        }
    });
    let res = reqwest::Client::new()
        .post(LC_GRAPHQL_ENDPOINT)
        .json(&req_body)
        .send()
        .await?;
    let parsed = res.json::<PromptResponse>().await?;

    Ok(parsed.data.question.content)
}