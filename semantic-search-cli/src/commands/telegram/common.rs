//! Common functions for telegram commands.

// Using static stickers (requires .PNG or .WEBP format):
// UploadStickerFile?
// GetStickerSet / CreateNewStickerSet?
// AddStickerToSet
// SendSticker or InlineQueryResultCachedSticker

// Using photos (requires .JPG or .WEBP format):
//
// SendPhoto or InlineQueryResult(Cached)Photo

use frankenstein::{client_reqwest::Bot, AddStickerToSetParams, AsyncTelegramApi, CreateNewStickerSetParams, Error, FileUpload, GetStickerSetParams, InputFile, InputSticker, StickerFormat, StickerType, UploadStickerFileParams, UploadStickerFileParamsBuilder, User};
use log::{debug, error, warn};

use super::BotConfig;
use crate::util::Database;

/// Ensure the files are uploaded, also updating the database and filtering out non .PNG or .WEBP files.
pub async fn upload_or_reuse(
    bot: &Bot,
    me: &User,
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

        let user_id = me.id;
        let sticker = InputFile::builder().path((&path).into()).build();
        let sticker_params = UploadStickerFileParams::builder()
            .sticker_format(StickerFormat::Static)
            .user_id(user_id)
            .sticker(sticker)
            .build();
        let uploaded = bot.upload_sticker_file(&sticker_params).await;

        if let Ok(uploaded) = uploaded {
            let file_id = uploaded.result.file_id;
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
async fn upload_sticker_set(bot: &Bot, me: &User, config: &BotConfig, sticker_ids: Vec<&str>) {
    let owner = config.owner;
    let prefix = &config.sticker_set;
    let Some(bot_name) = &me.username else {
        error!("Cannot upload stickers without a bot username.");
        return;
    };
    let name = format!("{prefix}_by_{bot_name}");

    let stickerset_params = GetStickerSetParams::builder().name(&name).build();
    match bot.get_sticker_set(&stickerset_params).await {
        Ok(sticker_set) => {} // Sticker set exists
        Err(error) => {
            match error {
                Error::Api(error) => {
                    let description = &error.description;
                    if description == "Bad Request: STICKERSET_INVALID" {
                        // Sticker set does not exist
                        create_sticker_set(bot, &name, owner, &sticker_ids).await;
                    } else {
                        error!("Failed to get sticker set - unexpected api error: {error:?}");
                        return;
                    }
                }
                error => {
                    error!("Failed to get sticker set - unexpected error: {error:?}");
                    return;
                }
            }
        }
    }

    for id in sticker_ids {
        let add_params = AddStickerToSetParams::builder()
            .user_id(owner)
            .name(&name)
            .sticker(sticker(id))
            .build();
        let result = bot.add_sticker_to_set(&add_params).await;
        if let Err(error) = result {
            error!("Failed to add sticker to set: {error}");
        } else {
            debug!("Added sticker to set: {id}");
        }
    }
}

/// Creates a sticker set with the given full name.
async fn create_sticker_set(bot: &Bot, name: &str, owner: u64, sticker_ids: &Vec<&str>) {
    let stickers: Vec<_> = sticker_ids.iter().map(|id| sticker(id)).collect();
    let create_params = CreateNewStickerSetParams::builder()
        .user_id(owner)
        .name(name)
        .title(name)
        .stickers(stickers)
        .sticker_type(StickerType::Regular)
        .build();
    let result = bot.create_new_sticker_set(&create_params).await;
    if let Err(error) = result {
        error!("Failed to create sticker set: {error}");
    } else {
        debug!("Created sticker set `{name}` with {} stickers", sticker_ids.len());
    }
}

/// Creates a sticker from sticker id.
fn sticker(sticker_id: &str) -> InputSticker {
    InputSticker::builder()
        .sticker(FileUpload::String(sticker_id.to_string()))
        .format(StickerFormat::Static)
        .emoji_list(vec!["😼".to_string()])
        .build()
}
