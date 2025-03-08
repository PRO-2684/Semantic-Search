//! Common functions for telegram commands.

// Using static stickers:
// UploadStickerFile?
// GetStickerSet / CreateNewStickerSet?
// AddStickerToSet
// SendSticker or InlineQueryResultCachedSticker

use std::{path::PathBuf, sync::Arc};

use futures_util::future;
use frankenstein::{client_reqwest::Bot, AddStickerToSetParams, AsyncTelegramApi, CreateNewStickerSetParams, Error, FileUpload, GetStickerSetParams, InputFile, InputSticker, StickerFormat, StickerType, UploadStickerFileParams, UploadStickerFileParamsBuilder, User};
use log::{debug, error, warn};
use image::{error::{ImageFormatHint, UnsupportedError, UnsupportedErrorKind}, imageops::FilterType, GenericImageView, ImageError, ImageFormat, ImageResult};
use tokio::sync::Mutex;

use super::BotConfig;
use crate::util::Database;

/// Ensure the files are uploaded, also updating the database.
pub async fn upload_or_reuse(
    bot: &Bot,
    me: &User,
    db: Arc<Mutex<Database>>,
    config: &BotConfig,
    pairs: Vec<(String, f32, Option<String>)>,
) -> Vec<(String, f32, String)> {
    let user_id = me.id;
    let tasks: Vec<_> = pairs.into_iter().map(|(path, confidence, file_id)| {
        let db = db.clone();
        async move { // This closure returns Some((path, confidence, file_id, is_new) if successful.
            // Already uploaded?
            if let Some(existing_id) = file_id {
                debug!("Sticker file already uploaded {path} with id {existing_id}");
                return Some((path, confidence, existing_id, false));
            }

            // Image conversion
            let (image, is_temp) = match convert_if_necessary(&path) {
                Ok((image, is_temp)) => {
                    (image, is_temp)
                }
                Err(e) => {
                    warn!("Failed to convert image: {e} for {path}");
                    return None;
                }
            };

            // Upload the sticker
            let sticker = InputFile::builder().path(image.clone()).build();
            let sticker_params = UploadStickerFileParams::builder()
                .sticker_format(StickerFormat::Static)
                .user_id(user_id)
                .sticker(sticker)
                .build();
            let uploaded = bot.upload_sticker_file(&sticker_params).await;
            if is_temp {
                std::fs::remove_file(&image).unwrap();
            }

            if let Ok(uploaded) = uploaded {
                let file_id = uploaded.result.file_id;
                let mut db = db.lock().await;
                match db.set_file_id(&path, &file_id).await {
                    Ok(true) => debug!("Uploaded sticker file {path} with id {file_id}"),
                    Ok(false) => warn!("Failed to update database: row affected mismatch for {path}"),
                    Err(e) => warn!("Failed to update database: {e} for {path}"),
                }
                Some((path, confidence, file_id, true))
            } else {
                warn!("Failed to upload sticker file: {path}");
                None
            }
        }
    }).collect();

    // Unzip the results
    let results = future::join_all(tasks).await.into_iter().filter_map(|x| x);
    let (results, is_new): (Vec<_>, Vec<_>) = results.map(
        |(path, confidence, file_id, is_new)| ((path, confidence, file_id), is_new)
    ).unzip();
    let new_stickers: Vec<_> = results.iter().zip(is_new.into_iter()).filter_map(
        |((path, confidence, file_id), is_new)| if is_new { Some(file_id) } else { None }
    ).collect();

    upload_sticker_set(&bot, me, config, new_stickers).await;
    results
}

/// Uploads to a sticker set. Creates one if not found.
async fn upload_sticker_set(bot: &Bot, me: &User, config: &BotConfig, sticker_ids: Vec<&String>) {
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
                        debug!("Sticker set does not exist: {name}, creating...");
                        create_sticker_set(bot, &name, owner, &sticker_ids).await;
                        return;
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

    let add_params: Vec<_> = sticker_ids.iter().map(|id| {
        AddStickerToSetParams::builder()
            .user_id(owner)
            .name(&name)
            .sticker(sticker(id))
            .build()
    }).collect();
    let results = futures_util::future::join_all(add_params.iter().map(|params| bot.add_sticker_to_set(params))).await;
    for (id, result) in sticker_ids.iter().zip(results) {
        if let Err(error) = result {
            error!("Failed to add sticker {id} to set: {error}");
        } else {
            debug!("Added sticker {id} to set");
        }
    }
}

/// Creates a sticker set with the given full name.
async fn create_sticker_set(bot: &Bot, name: &str, owner: u64, sticker_ids: &Vec<&String>) {
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

/// Convert the image if necessary, returning the new path and whether a temporary file was created.
fn convert_if_necessary(path: &str) -> ImageResult<(PathBuf, bool)> {
    // Requirements:
    // 1. .PNG or .WEBP format
    // 2. One side must be 512px, the other side equal or less than 512px

    // We only accept JPEG, PNG, and WEBP formats.
    const ACCEPTED_EXTENSIONS: [&str; 4] = ["jpeg", "jpg", "png", "webp"];
    let path = PathBuf::from(path);
    let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    if !ACCEPTED_EXTENSIONS.contains(&ext.as_str()) {
        let format = match ImageFormat::from_extension(&ext) {
            Some(format) => ImageFormatHint::Exact(format),
            None => ImageFormatHint::Name(ext.to_string()),
        };
        let kind = UnsupportedErrorKind::Format(format.clone());
        return Err(ImageError::Unsupported(UnsupportedError::from_format_and_kind(
            format, kind
        )));
    }

    let image = image::open(&path)?; // .into_rgba16();
    let (width, height) = image.dimensions();

    // Use the original image if it meets the requirements
    let one_side_512 = width == 512 || height == 512;
    let both_leq_512 = width <= 512 && height <= 512;
    if one_side_512 && both_leq_512 {
        debug!("Image already meets requirements: {}", path.display());
        return Ok((path, false));
    }

    // Call resize_to_fill, which automatically fits for us
    let new_path = path.with_extension("tmp.png");
    let resized = image.resize_to_fill(512, 512, FilterType::Lanczos3);
    debug!("Resized image: {} to {}", path.display(), new_path.display());
    resized.save(&new_path)?;

    Ok((new_path, true))
}
