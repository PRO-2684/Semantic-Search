//! Common functions for telegram commands.

// Init stickers:
// UploadStickerFile?
// GetStickerSet / CreateNewStickerSet?
// AddStickerToSet
// DeleteStickerFromSet

// Sending stickers:
// SendSticker or InlineQueryResultCachedSticker

use std::path::PathBuf;

use frankenstein::{
    client_reqwest::Bot, AddStickerToSetParams, AsyncTelegramApi, CreateNewStickerSetParams,
    DeleteStickerFromSetParams, Error, FileUpload, GetStickerSetParams, InputFile, InputSticker,
    StickerFormat, StickerSet, StickerType, UploadStickerFileParams, User,
};
use futures_util::{future, StreamExt};
use image::{
    error::{ImageFormatHint, UnsupportedError, UnsupportedErrorKind},
    imageops::FilterType,
    GenericImageView, ImageError, ImageFormat, ImageResult,
};
use log::{debug, error, warn};

use super::{BotConfig, BotResult};
use crate::util::Database;

/// Initialize stickers.
pub async fn init_stickers(
    bot: &Bot,
    me: &User,
    db: &mut Database,
    config: &BotConfig,
) -> anyhow::Result<()> {
    let Some(bot_name) = &me.username else {
        anyhow::bail!("Cannot initialize stickers without a bot username.");
    };
    let sticker_set_name = format!("{}_by_{}", config.sticker_set, bot_name);
    let get_params = GetStickerSetParams::builder()
        .name(&sticker_set_name)
        .build();

    // Check if the sticker set exists
    let mut paths_iter = db.iter_file_ids().filter_map(|row| {
        future::ready(
            // Only yields path on Ok variants with missing file ids
            match row {
                Ok((path, None)) => Some(path),
                Ok((path, Some(file_id))) => {
                    debug!("Sticker {path} already uploaded: {file_id}");
                    None
                }
                Err(e) => {
                    warn!("Failed to read database: {e}");
                    None
                }
            },
        )
    });
    let sticker_set = get_sticker_set(bot, &get_params).await;
    let mut collected_paths = Vec::new();

    if let Some(sticker_set) = sticker_set {
        // Empty the sticker set
        debug!("Sticker set found: {sticker_set_name}, emptying...");
        empty_sticker_set(bot, sticker_set).await?;
    } else {
        // If the sticker set does not exist, create it with one sticker
        debug!("Sticker set not found: {sticker_set_name}, creating...");
        let Some(path) = paths_iter.next().await else {
            anyhow::bail!("No stickers found in the database.");
        };
        let Ok(file_id) = upload_sticker_file(bot, &path, me.id).await else {
            anyhow::bail!("Failed to upload sticker file: {path}");
        };
        create_sticker_set(bot, &sticker_set_name, me.id, &vec![&file_id]).await?;
        collected_paths.push(path);
    }

    // Upload the rest of the stickers
    debug!("Uploading stickers...");
    while let Some(path) = paths_iter.next().await {
        // NOTE: This shouldn't be done in parallel, as the stickers must be uploaded in order
        let Ok(file_id) = upload_sticker_file(bot, &path, me.id).await else {
            anyhow::bail!("Failed to upload sticker file: {path}");
        };
        let add_params = AddStickerToSetParams::builder()
            .user_id(me.id)
            .name(&sticker_set_name)
            .sticker(sticker(&file_id))
            .build();
        let result = bot.add_sticker_to_set(&add_params).await;
        if let Err(error) = result {
            error!("Failed to add sticker {path} to set: {error}");
        } else {
            debug!("Added sticker {path} to set");
            collected_paths.push(path);
        }
    }
    drop(paths_iter);

    // Get file ids using GetStickerSet
    let sticker_set = bot.get_sticker_set(&get_params).await?.result;
    // Take the last `paths.len()` stickers
    let skip = sticker_set.stickers.len() - collected_paths.len();
    let file_ids = sticker_set
        .stickers
        .iter()
        .skip(skip)
        .map(|sticker| &sticker.file_id);

    // Update database
    debug!("Updating database...");
    for (path, file_id) in collected_paths.iter().zip(file_ids) {
        match db.set_file_id(path, file_id).await {
            Ok(true) => debug!("Updated database with file id for {path}"),
            Ok(false) => warn!("Failed to update database: row affected mismatch for {path}"),
            Err(e) => warn!("Failed to update database: {e} for {path}"),
        }
    }

    // Empty the sticker set
    debug!("Emptying sticker set...");
    empty_sticker_set(bot, sticker_set).await?;

    Ok(())
}

/// Check if the sticker set exists, returning the sticker set if found.
async fn get_sticker_set(bot: &Bot, get_params: &GetStickerSetParams) -> Option<StickerSet> {
    match bot.get_sticker_set(get_params).await {
        Ok(result) => Some(result.result),
        Err(error) => {
            match error {
                Error::Api(error) => {
                    let description = &error.description;
                    if description != "Bad Request: STICKERSET_INVALID" {
                        error!("Failed to get sticker set - unexpected api error: {error:?}");
                    }
                }
                error => {
                    error!("Failed to get sticker set - unexpected error: {error:?}");
                }
            }
            None
        }
    }
}

/// Upload a sticker file.
async fn upload_sticker_file(bot: &Bot, path: &str, user_id: u64) -> Result<String, String> {
    // Image conversion
    let (image, is_temp) = match convert_if_necessary(path) {
        Ok((image, is_temp)) => (image, is_temp),
        Err(e) => {
            return Err(format!("Failed to convert image: {e} for {path}"));
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
        debug!("Uploaded sticker file {path} with id {file_id}");
        Ok(file_id)
    } else {
        Err(format!("Failed to upload sticker file: {path}"))
    }
}

/// Empty the sticker set.
pub async fn empty_sticker_set(bot: &Bot, sticker_set: StickerSet) -> BotResult<Vec<String>> {
    let file_ids: Vec<_> = sticker_set
        .stickers
        .into_iter()
        .map(|sticker| sticker.file_id)
        .collect();
    let delete_params: Vec<_> = file_ids
        .iter()
        .map(|id| DeleteStickerFromSetParams::builder().sticker(id).build())
        .collect();
    let results = futures_util::future::join_all(
        delete_params
            .iter()
            .map(|params| bot.delete_sticker_from_set(params)),
    )
    .await;
    for (id, result) in file_ids.iter().zip(results) {
        if let Err(error) = result {
            error!("Failed to delete sticker {id} from set: {error}");
        } else {
            debug!("Deleted sticker {id} from set");
        }
    }

    Ok(file_ids)
}

/// Create a sticker set with the given full name.
async fn create_sticker_set(
    bot: &Bot,
    name: &str,
    owner: u64,
    file_ids: &Vec<&String>,
) -> BotResult<()> {
    let stickers: Vec<_> = file_ids.iter().map(|id| sticker(id)).collect();
    let create_params = CreateNewStickerSetParams::builder()
        .user_id(owner)
        .name(name)
        .title(name)
        .stickers(stickers)
        .sticker_type(StickerType::Regular)
        .build();
    let result = bot.create_new_sticker_set(&create_params).await;
    result.map(|_| ())
}

/// Create a sticker from file id.
fn sticker(file_id: &str) -> InputSticker {
    InputSticker::builder()
        .sticker(FileUpload::String(file_id.to_string()))
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
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if !ACCEPTED_EXTENSIONS.contains(&ext.as_str()) {
        let format = ImageFormat::from_extension(&ext)
            .map_or(ImageFormatHint::Name(ext), |format| {
                ImageFormatHint::Exact(format)
            });
        let kind = UnsupportedErrorKind::Format(format.clone());
        return Err(ImageError::Unsupported(
            UnsupportedError::from_format_and_kind(format, kind),
        ));
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
    let new_path = path.with_extension("tmp.webp");
    let resized = image.resize_to_fill(512, 512, FilterType::Lanczos3);
    debug!(
        "Resized image: {} to {}",
        path.display(),
        new_path.display()
    );
    resized.save(&new_path)?;

    Ok((new_path, true))
}
