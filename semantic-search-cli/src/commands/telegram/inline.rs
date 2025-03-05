//! Module for handling inline queries.

use super::{ApiClient, Database, ThrottledBot};
use semantic_search::Embedding;
use std::sync::Arc;
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    prelude::{Requester, ResponseResult},
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Update,
    },
};
use tokio::sync::Mutex;

/// Handles inline queries.
pub async fn inline_handler(
    bot: ThrottledBot,
    query: InlineQuery,
    db: Arc<Mutex<Database>>,
    api: ApiClient,
) -> ResponseResult<()> {
    let InlineQuery { query: query_str, id: query_id, .. } = query;
    let query_str = query_str.trim();
    if query_str.is_empty() {
        let results = vec![
            article("1", "😸 Meow!", "Continue typing to search...", "Continue typing to search..."),
        ];
        bot.answer_inline_query(query_id, results).await?;
        Ok(())
    } else {
        let mut db = db.lock().await;
        handle_query(bot, query_str, query_id, &mut db, api).await;
        Ok(())
    }
}

/// Handles non-empty inline queries.
async fn handle_query(
    bot: ThrottledBot,
    query_str: &str,
    query_id: String,
    db: &mut Database,
    api: ApiClient,
) {
    let Ok(raw_embedding) = api.embed(query_str).await else {
        let _ = bot.answer_inline_query(query_id, vec![
            article("1", "😿 Error", "Failed to embed the query.", "Failed to embed the query."),
        ]).await;
        return;
    };
    let embedding: Embedding = raw_embedding.into();
    let results = db.search(5, &embedding);
    let Ok(results) = results.await else {
        let _ = bot.answer_inline_query(query_id, vec![
            article("1", "😿 Error", "Failed to search the database.", "Failed to search the database."),
        ]).await;
        return;
    };
    if results.is_empty() {
        let _ = bot.answer_inline_query(query_id, vec![
            article("1", "😿 No results", "No results found.", "No results found."),
        ]).await;
        return;
    }
    let articles: Vec<InlineQueryResult> = results
        .into_iter()
        .enumerate()
        .map(|(index, (path, similarity))| {
            let percent = similarity * 100.0;
            article(
                index.to_string(),
                format!("#{}: {percent:.2}%", index + 1),
                format!("🐾 {percent:.2}: {path}"),
                path,
            )
        })
        .collect();
    let _ = bot.answer_inline_query(query_id, articles).await;
}

fn article<S1, S2, S3, S4>(id: S1, title: S2, description: S3, content: S4) -> InlineQueryResult
where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
    S4: Into<String>,
{
    InlineQueryResult::Article(InlineQueryResultArticle::new(
        id,
        title,
        InputMessageContent::Text(InputMessageContentText::new(content)),
    ).description(description))
}
