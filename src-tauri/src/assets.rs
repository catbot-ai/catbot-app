use image::ImageReader;
use reqwest::Client;
use std::io::Cursor;
use tauri::image::Image;

use crate::{jup::TokenId, ray::get_token_logo_url_by_mint_address};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufReader;

pub fn read_local_image(file_path: &str) -> Result<Image> {
    // Open the file
    let file = File::open(file_path).context("Failed to open file")?;
    let reader = BufReader::new(file);

    // Decode the image
    let img = ImageReader::new(reader)
        .with_guessed_format() // Guess the image format based on the file content
        .context("Failed to guess image format")?
        .decode()
        .context("Failed to decode image")?;

    // Convert the image to RGBA8 format
    let rgba_img = img.to_rgba8();
    let img_vec = rgba_img.to_vec();

    // Create and return your custom Image type
    let image = Image::new_owned(img_vec, rgba_img.width(), rgba_img.height());
    Ok(image)
}

pub async fn fetch_token_image(token_id: &TokenId) -> anyhow::Result<Image> {
    let token_logo_url = get_token_logo_url_by_mint_address(&token_id.to_string());
    let logo_url = format!("https://img-v1.raydium.io/icon/{token_logo_url}.png");

    let image = fetch_image(&logo_url).await?;

    Ok(image.to_owned())
}

pub async fn fetch_image(url: &str) -> anyhow::Result<Image> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let img_vec = img.to_rgba8().to_vec();
    let image = Image::new_owned(img_vec, img.width(), img.height());

    Ok(image)
}

pub async fn fetch_and_set_icon(
    url: &str,
    tray: &tauri::tray::TrayIcon,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = fetch_image(url).await?;

    tray.set_icon(Some(image))?;

    Ok(())
}
