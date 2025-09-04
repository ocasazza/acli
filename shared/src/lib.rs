use serde::{Deserialize, Serialize};

/// A post with a title, text, and author.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Post {
    /// The title of the post.
    pub title: String,
    /// The text of the post.
    pub text: String,
    /// The name of the author of the post.
    pub author_name: String,
}
