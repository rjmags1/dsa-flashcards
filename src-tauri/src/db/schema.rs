table! {
    question (id) {
        id -> Integer,
        question_number -> Integer,
        title -> Text,
        title_slug -> Text,
        prompt -> Text,
        difficulty -> Text,
        fetched -> Bool,
    }
}

table! {
    question_topic (id) {
        id -> Integer,
        question_number -> Integer,
        topic -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    question,
    question_topic,
);
