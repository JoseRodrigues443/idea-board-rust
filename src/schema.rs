table! {
    likes (id) {
        id -> Uuid,
        created_at -> Timestamp,
        idea_id -> Uuid,
    }
}

table! {
    ideas (id) {
        id -> Uuid,
        created_at -> Timestamp,
        message -> Text,
        image -> Text,
    }
}

joinable!(likes -> ideas (idea_id));

allow_tables_to_appear_in_same_query!(
    likes,
    ideas,
);
