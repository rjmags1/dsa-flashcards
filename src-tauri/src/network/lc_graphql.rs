pub const LC_GRAPHQL_ENDPOINT: &str = "https://leetcode.com/graphql";

pub const LIST_LEN_LIMIT: i32 = 2500;

pub const Q_LIST_QUERY: &str = "query \
problemsetQuestionList(\
    $categorySlug: String, \
    $limit: Int, \
    $filters: QuestionListFilterInput\
) { \
    problemsetQuestionList: questionList( \
        categorySlug: $categorySlug \
        limit: $limit \
        filters: $filters \
    ) { \
        questions: data { \
            difficulty \
            questionId \
            title \
            titleSlug \
            topicTags { name id slug }  \
        } \
    }\
}"; 


pub const Q_PROMPT_QUERY: &str = "query \
questionData($titleSlug: String!) { \
    question(titleSlug: $titleSlug) { \
        content \
    }\
}";