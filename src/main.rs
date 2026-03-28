#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod gui;
mod i18n;
mod process;
mod scanner;
mod utils;

use anyhow::Result;
use eframe::egui;
use image::imageops::FilterType;
use std::io::Cursor;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use crate::app::RssStringsApp;

fn main() -> Result<()> {
    init_tracing();
    utils::admin::ensure_admin()?;

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_min_inner_size([960.0, 640.0])
        .with_app_id("RSS-strings");
    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "RSS-strings",
        native_options,
        Box::new(|cc| Ok(Box::new(RssStringsApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init();
}

fn load_app_icon() -> Option<egui::IconData> {
    // Use embedded ICO; pick the largest frame so the window icon isn't tiny.
    let bytes = include_bytes!("../RSS-strings.ico");
    let mut cursor = Cursor::new(bytes.as_ref());
    let icon_dir = ico::IconDir::read(&mut cursor).ok()?;
    let entry = icon_dir
        .entries()
        .iter()
        .max_by_key(|e| e.width() * e.height())
        .cloned()?;
    let image = entry.decode().ok()?;

    let target_size = 256u32;
    let rgba = if image.width() >= target_size && image.height() >= target_size {
        image.rgba_data().to_vec()
    } else {
        let base =
            image::RgbaImage::from_raw(image.width(), image.height(), image.rgba_data().to_vec())?;
        image::imageops::resize(&base, target_size, target_size, FilterType::Lanczos3).into_raw()
    };
    Some(egui::IconData {
        rgba,
        width: target_size,
        height: target_size,
    })
}
