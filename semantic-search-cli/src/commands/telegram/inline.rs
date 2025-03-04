//! Module for handling inline queries.

use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    prelude::{Requester, ResponseResult},
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Update,
    },
};

use super::ThrottledBot;

/// Handles inline queries.
pub async fn inline_handler(bot: ThrottledBot, q: InlineQuery) -> ResponseResult<()> {
    answer_inline(bot, q).await
}

async fn answer_inline(bot: ThrottledBot, query: InlineQuery) -> ResponseResult<()> {
    // TODO: Implement the inline query handler.
    let results = vec![
        article("1", "Hello, world! 1", "Hello, world! 1"),
        article("2", "Hello, world! 2", "Hello, world! 2"),
    ];

    bot.answer_inline_query(query.id, results).await?;

    Ok(())
}

fn article<S1, S2, S3>(id: S1, title: S2, content: S3) -> InlineQueryResult
where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
{
    InlineQueryResult::Article(InlineQueryResultArticle::new(
        id,
        title,
        InputMessageContent::Text(InputMessageContentText::new(content)),
    ))
}
