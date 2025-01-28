use image::ImageReader;
use reqwest::Client;
use std::io::Cursor;
use tauri::image::Image;

pub async fn fetch_and_set_icon(
    url: &str,
    tray: &tauri::tray::TrayIcon,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;

    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let img_vec = img.to_rgba8().to_vec();
    let image = Image::new(&img_vec, img.width(), img.height());

    tray.set_icon(Some(image))?;

    Ok(())
}
