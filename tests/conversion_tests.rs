use std::fs;
use image::{ImageBuffer, Rgba};
use kisayolluk::config::settings::{AppSettings, CollisionStrategy};
use kisayolluk::conversion::engine::ConversionEngine;
use kisayolluk::conversion::output_manager::OutputManager;
use kisayolluk::files::file_type::SupportedImageFormat;

#[test]
fn test_target_formats_excludes_source() {
    let png_targets = SupportedImageFormat::targets_for(Some(SupportedImageFormat::Png));
    assert!(!png_targets.contains(&SupportedImageFormat::Png));
    assert!(png_targets.contains(&SupportedImageFormat::Jpeg));
    assert!(png_targets.contains(&SupportedImageFormat::Webp));
    assert!(png_targets.contains(&SupportedImageFormat::Bmp));
    assert!(png_targets.contains(&SupportedImageFormat::Tiff));

    let jpg_targets = SupportedImageFormat::targets_for(Some(SupportedImageFormat::Jpeg));
    assert!(!jpg_targets.contains(&SupportedImageFormat::Jpeg));
    assert!(jpg_targets.contains(&SupportedImageFormat::Png));
}

#[test]
fn test_output_manager_collision_strategies() {
    let temp_dir = std::env::temp_dir().join("kisayolluk_test_output");
    let _ = fs::create_dir_all(&temp_dir);

    let test_src = temp_dir.join("ornek_foto.png");
    fs::write(&test_src, b"dummy").unwrap();

    // Direct path (no collision)
    let out_1 = OutputManager::determine_output_path(
        &test_src,
        SupportedImageFormat::Jpeg,
        None,
        CollisionStrategy::AddSuffix,
    ).unwrap();
    assert_eq!(out_1, temp_dir.join("ornek_foto.jpg"));

    // Create that file to induce collision
    fs::write(&out_1, b"existing").unwrap();

    // Suffix strategy -> should create ornek_foto_1.jpg
    let out_2 = OutputManager::determine_output_path(
        &test_src,
        SupportedImageFormat::Jpeg,
        None,
        CollisionStrategy::AddSuffix,
    ).unwrap();
    assert_eq!(out_2, temp_dir.join("ornek_foto_1.jpg"));

    // Overwrite strategy -> returns direct target
    let out_over = OutputManager::determine_output_path(
        &test_src,
        SupportedImageFormat::Jpeg,
        None,
        CollisionStrategy::Overwrite,
    ).unwrap();
    assert_eq!(out_over, out_1);

    // Skip strategy -> errors
    let err_skip = OutputManager::determine_output_path(
        &test_src,
        SupportedImageFormat::Jpeg,
        None,
        CollisionStrategy::Skip,
    );
    assert!(err_skip.is_err());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_real_image_conversion() {
    let temp_dir = std::env::temp_dir().join("kisayolluk_test_images");
    let _ = fs::create_dir_all(&temp_dir);

    // 1. Create a 100x80 test PNG image with transparent pixels
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(100, 80);
    for (x, _y, pixel) in img.enumerate_pixels_mut() {
        if x < 50 {
            *pixel = Rgba([255, 0, 0, 255]); // Opaque Red
        } else {
            *pixel = Rgba([0, 255, 0, 128]); // Semi-transparent Green
        }
    }

    let png_path = temp_dir.join("sample.png");
    img.save(&png_path).unwrap();
    assert!(png_path.exists());

    let mut settings = AppSettings::default();
    settings.conversion.jpeg_quality = 92;
    settings.conversion.jpeg_background_color = [255, 255, 255]; // White

    // 2. Convert PNG -> JPEG
    let jpg_path = ConversionEngine::execute_conversion(
        &png_path,
        SupportedImageFormat::Jpeg,
        &settings,
    ).unwrap();

    assert!(jpg_path.exists());
    let decoded_jpg = image::open(&jpg_path).unwrap();
    assert_eq!(decoded_jpg.width(), 100);
    assert_eq!(decoded_jpg.height(), 80);

    // 3. Convert PNG -> WEBP
    let webp_path = ConversionEngine::execute_conversion(
        &png_path,
        SupportedImageFormat::Webp,
        &settings,
    ).unwrap();

    assert!(webp_path.exists());
    let decoded_webp = image::open(&webp_path).unwrap();
    assert_eq!(decoded_webp.width(), 100);
    assert_eq!(decoded_webp.height(), 80);

    // 4. Convert PNG -> BMP
    let bmp_path = ConversionEngine::execute_conversion(
        &png_path,
        SupportedImageFormat::Bmp,
        &settings,
    ).unwrap();

    assert!(bmp_path.exists());
    let decoded_bmp = image::open(&bmp_path).unwrap();
    assert_eq!(decoded_bmp.width(), 100);
    assert_eq!(decoded_bmp.height(), 80);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

