//! Common functions for telegram commands.

// Using static stickers (requires .PNG or .WEBP format):
// UploadStickerFile?
// GetStickerSet / CreateNewStickerSet?
// AddStickerToSet
// SendSticker or InlineQueryResultCachedSticker

// Using photos (requires .JPG or .WEBP format):
//
// SendPhoto or InlineQueryResult(Cached)Photo

use log::{debug, error, warn};
use teloxide::{
    payloads::UploadStickerFile, prelude::Requester, types::{InputFile, InputSticker, Me, StickerFormat, UserId}, ApiError, RequestError
};

use super::{BotConfig, WrappedBot};
use crate::util::Database;

/// Ensure the files are uploaded, also updating the database and filtering out non .PNG or .WEBP files.
pub async fn upload_or_reuse(
    bot: &WrappedBot,
    me: &Me,
    db: &mut Database,
    config: &BotConfig,
    pairs: Vec<(String, f32, Option<String>)>,
) -> Vec<(String, f32, String)> {
    let mut results = Vec::with_capacity(pairs.len());
    // Newly uploaded stickers
    let mut new = Vec::new();

    for (path, confidence, file_id) in pairs {
        // Already uploaded
        if let Some(file_id) = file_id {
            results.push((path, confidence, file_id));
            continue;
        }

        // Not expected format
        let extension = path.split('.').last().unwrap_or_default().to_lowercase();
        if extension != "png" && extension != "webp" {
            warn!("Non-PNG or non-WEBP file detected: {}", path);
            continue;
        }

        let user_id = bot.get_me().await.unwrap().user.id;
        let sticker = InputFile::file(&path);
        let uploaded = bot
            .upload_sticker_file(user_id, sticker, StickerFormat::Static)
            .await;
        if let Ok(uploaded) = uploaded {
            let file_id = uploaded.id;
            match db.set_file_id(&path, &file_id).await {
                Ok(true) => debug!("Uploaded sticker file: {}", path),
                Ok(false) => warn!("Failed to update database: row affected mismatch for {path}"),
                Err(e) => warn!("Failed to update database: {e} for {path}"),
            }
            results.push((path, confidence, file_id.clone()));
            new.push(file_id);
        } else {
            warn!("Failed to upload sticker file: {path}");
        }
    }

    let sticker_ids = new.iter().map(|file_id| file_id.as_str()).collect();
    upload_sticker_set(bot, me, config, sticker_ids).await;

    results
}

/// Uploads to a sticker set. Creates one if not found.
async fn upload_sticker_set(bot: &WrappedBot, me: &Me, config: &BotConfig, sticker_ids: Vec<&str>) {
    let owner = config.owner;
    let prefix = &config.sticker_set;
    let Some(bot_name) = &me.username else {
        error!("Cannot upload stickers without a bot username.");
        return;
    };
    let name = format!("{prefix}_by_{bot_name}");

    match bot.get_sticker_set(&name).await {
        Ok(sticker_set) => {} // Sticker set exists
        Err(error) => {
            match error {
                RequestError::Api(error) => {
                    match error {
                        ApiError::InvalidStickersSet => {
                            // Sticker set does not exist
                            create_sticker_set(bot, &name, owner, &sticker_ids).await;
                        }
                        _ => {
                            error!("Failed to get sticker set - unexpected ApiError: {error:?}");
                            return;
                        }
                    }
                }
                error => {
                    error!("Failed to get sticker set - unexpected RequestError: {error:?}");
                    return;
                }
            }
        }
    }

    for id in sticker_ids {
        let result = bot.add_sticker_to_set(UserId(owner), &name, sticker(id)).await;
        if let Err(error) = result {
            error!("Failed to add sticker to set: {error}");
        } else {
            debug!("Added sticker to set: {id}");
        }
    }
}

/// Creates a sticker set with the given full name.
async fn create_sticker_set(bot: &WrappedBot, name: &str, owner: u64, sticker_ids: &Vec<&str>) {
    let stickers: Vec<_> = sticker_ids.iter().map(|id| sticker(id)).collect();
    let result = bot.create_new_sticker_set(UserId(owner), name, name, stickers, StickerFormat::Static).await;
    if let Err(error) = result {
        error!("Failed to create sticker set: {error}");
    } else {
        debug!("Created sticker set `{name}` with {} stickers", sticker_ids.len());
    }
}

/// Creates a sticker from sticker id.
fn sticker(sticker_id: &str) -> InputSticker {
    InputSticker {
        sticker: InputFile::file_id(sticker_id),
        emoji_list: vec!["😼".to_string()],
        mask_position: None,
        keywords: vec![]
    }
}
