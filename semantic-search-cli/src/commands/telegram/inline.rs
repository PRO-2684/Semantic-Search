//! Module for handling inline queries.

use super::{ApiClient, BotConfig, Database, upload_or_reuse};
use frankenstein::{client_reqwest::Bot, InlineQuery, User};
use log::info;
use semantic_search::Embedding;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles inline queries.
pub async fn inline_handler(
    bot: &Bot,
    me: &User,
    query: InlineQuery,
    db: &mut Database,
    api: &ApiClient,
    config: &BotConfig,
) -> ResponseResult<()> {
    let InlineQuery { query: query_str, id: query_id, .. } = query;
    let query_str = query_str.trim();
    if query_str.is_empty() {
        let results = vec![
            article("1", "😸 Meow!", "Continue typing to search..."),
        ];
        bot.answer_inline_query(query_id, results).await?;
        Ok(())
    } else {
        let mut db = db.lock().await;
        handle_query(&bot, &me, query_str, query_id, &mut db, &api, config).await;
        Ok(())
    }
}

/// Handles non-empty inline queries.
async fn handle_query(
    bot: &WrappedBot,
    me: &Me,
    query_str: &str,
    query_id: String,
    db: &mut Database,
    api: &ApiClient,
    config: &BotConfig,
) {
    info!("Handling inline query: {}", query_str);
    let Ok(raw_embedding) = api.embed(query_str).await else {
        let _ = bot.answer_inline_query(query_id, vec![
            article("1", "😿 Error", "Failed to embed the query."),
        ]).await;
        return;
    };
    let embedding: Embedding = raw_embedding.into();
    let results = db.search_with_id(5, &embedding);
    let Ok(results) = results.await else {
        let _ = bot.answer_inline_query(query_id, vec![
            article("1", "😿 Error", "Failed to search the database."),
        ]).await;
        return;
    };
    let results = upload_or_reuse(bot, me, db, config, results).await;
    if results.is_empty() {
        let _ = bot.answer_inline_query(query_id, vec![
            article("1", "😿 No results", "No results found."),
        ]).await;
        return;
    }
    let stickers: Vec<InlineQueryResult> = results
        .into_iter()
        .enumerate()
        .map(|(index, (path, similarity, file_id))| {
            let percent = similarity * 100.0;
            sticker(
                index.to_string(),
                file_id,
            )
        })
        .collect();
    let answer = bot.answer_inline_query(query_id, stickers).await;
    if let Err(e) = answer {
        info!("Failed to answer inline query: {e}");
    }
}

/// Creates an article inline query result.
fn article(id: &str, title: &str, content: &str) -> InlineQueryResult {
    InlineQueryResult::Article(InlineQueryResultArticle::new(
        id,
        title,
        InputMessageContent::Text(InputMessageContentText::new(content)),
    ).description(content))
}

/// Creates an sticker inline query result.
fn sticker(id: String, file_id: String) -> InlineQueryResult {
    InlineQueryResult::CachedSticker(
        InlineQueryResultCachedSticker::new(id, file_id)
    )
}
