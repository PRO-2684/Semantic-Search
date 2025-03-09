//! Module for handling inline queries.

use super::{ApiClient, BotConfig, BotResult, Database};
use frankenstein::{
    client_reqwest::Bot, AnswerInlineQueryParams, AsyncTelegramApi, InlineQuery, InlineQueryResult,
    InlineQueryResultArticle, InlineQueryResultCachedSticker, InputMessageContent,
    InputTextMessageContent,
};
use log::info;
use semantic_search::Embedding;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles inline queries.
pub async fn inline_handler(
    bot: &Bot,
    query: InlineQuery,
    db: Arc<Mutex<Database>>,
    api: &ApiClient,
    config: &BotConfig,
) -> BotResult<()> {
    let InlineQuery {
        query: query_str,
        id: query_id,
        ..
    } = query;
    let query_str = query_str.trim();
    if query_str.is_empty() {
        bot.answer_inline_query(&text_query_params(
            &query_id,
            "Meow!",
            "Keep paw-typing to sniff out the purr-fect meme... 😸",
        ))
        .await?;
    } else {
        handle_query(bot, query_str, query_id, db, api, config).await?;
    }
    Ok(())
}

/// Handles non-empty inline queries.
async fn handle_query(
    bot: &Bot,
    query_str: &str,
    query_id: String,
    db: Arc<Mutex<Database>>,
    api: &ApiClient,
    config: &BotConfig,
) -> BotResult<()> {
    info!("Handling inline query: {}", query_str);
    let Ok(raw_embedding) = api.embed(query_str).await else {
        bot.answer_inline_query(&text_query_params(
            &query_id,
            "😿 Error",
            "Failed to embed the query.",
        ))
        .await?;
        return Ok(());
    };
    let embedding: Embedding = raw_embedding.into();
    let results = {
        let mut db = db.lock().await;
        db.search_with_id(config.num_results, &embedding).await
    };
    let Ok(results) = results else {
        bot.answer_inline_query(&text_query_params(
            &query_id,
            "😿 Error",
            "Failed to search the database.",
        ))
        .await?;
        return Ok(());
    };
    if results.is_empty() {
        bot.answer_inline_query(&text_query_params(
            &query_id,
            "😿 No results",
            "No results found.",
        ))
        .await?;
        return Ok(());
    }
    let stickers: Vec<InlineQueryResult> = results
        .into_iter()
        .enumerate()
        .map(|(index, (_path, _similarity, file_id))| sticker(index.to_string(), file_id))
        .collect();
    let answer_params = AnswerInlineQueryParams::builder()
        .inline_query_id(query_id)
        .results(stickers)
        .build();
    bot.answer_inline_query(&answer_params).await?;
    Ok(())
}

/// Creates an answer inline query parameters.
fn text_query_params(id: &str, title: &str, content: &str) -> AnswerInlineQueryParams {
    let message_content = InputMessageContent::Text(
        InputTextMessageContent::builder()
            .message_text(content)
            .build(),
    );
    let article = InlineQueryResult::Article(
        InlineQueryResultArticle::builder()
            .id("1")
            .title(title)
            .input_message_content(message_content)
            .description(content)
            .build(),
    );
    AnswerInlineQueryParams::builder()
        .inline_query_id(id)
        .results(vec![article])
        .build()
}

/// Creates an sticker inline query result.
fn sticker(id: String, file_id: String) -> InlineQueryResult {
    InlineQueryResult::Sticker(
        InlineQueryResultCachedSticker::builder()
            .id(id)
            .sticker_file_id(file_id)
            .build(),
    )
}
