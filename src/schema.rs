// @generated automatically by Diesel CLI.

diesel::table! {
    history (id) {
        id -> Integer,
        query -> Text,
    }
}
