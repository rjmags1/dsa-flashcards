use serde_json::json;
use crate::network::structs::{QuestionList, QuestionListResponse, PromptResponse};
use crate::network::lc_graphql::*;

pub async fn fetch_all_lc_questions() -> 
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

pub async fn fetch_lc_question_prompt(
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



///////////////////////////////////////
////// ----- UNIT TESTS -------- //////
///////////////////////////////////////
#[cfg(test)]
mod test {
    // note: these unit tests mainly serve as a quick check for breakage
    //       from the Leetcode graphql API

    use super::*;
    use regex::Regex;
    
    #[tokio::test]
    async fn fetch_all_lc_questions_test() {
        let fetched_question_list_res = fetch_all_lc_questions().await;
        assert!(fetched_question_list_res.is_ok());

        let fetched_question_list = fetched_question_list_res.unwrap();
        assert!(fetched_question_list.questions.len() > 0);
    }

    #[tokio::test]
    async fn fetch_lc_question_prompt_test() {
        let mock_slug = "two-sum".to_string();
        let fetched_prompt_res = fetch_lc_question_prompt(&mock_slug).await;
        assert!(fetched_prompt_res.is_ok());
        let fetched_prompt = fetched_prompt_res.unwrap();
        assert!(contains_html_tags(fetched_prompt));
    }

    pub fn contains_html_tags(s: String) -> bool {
        let re = Regex::new(r"</?[a-z][\s\S]*>").unwrap();
        re.is_match(&s)
    }
}