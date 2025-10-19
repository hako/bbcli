use image::{DynamicImage, ImageReader, Rgba, RgbaImage};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Simple in-memory image cache
pub struct ImageCache {
    cache: HashMap<String, DynamicImage>,
    max_size: usize,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 50, // Keep up to 50 images in cache
        }
    }

    /// Get image from cache or download it
    pub fn get_or_load(&mut self, url: &str) -> DynamicImage {
        // Check cache first
        if let Some(img) = self.cache.get(url) {
            return img.clone();
        }

        // Try to download
        match Self::download_image(url) {
            Ok(img) => {
                // Add to cache (simple, no LRU for now)
                if self.cache.len() >= self.max_size {
                    // Clear cache if too large (simple strategy)
                    self.cache.clear();
                }
                self.cache.insert(url.to_string(), img.clone());
                img
            }
            Err(_) => {
                // Return BBC logo placeholder on error
                Self::create_bbc_logo()
            }
        }
    }

    /// Download image from URL
    fn download_image(url: &str) -> anyhow::Result<DynamicImage> {
        // Use BBC-compatible user agent and timeout
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Linux; Android 10; SM-A307G) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let response = client.get(url).send()?;
        let bytes = response.bytes()?;
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        Ok(img)
    }

    /// Create BBC logo placeholder (red rectangle with white BBC text effect)
    pub fn create_bbc_logo() -> DynamicImage {
        let width = 240;
        let height = 135;

        let mut img = RgbaImage::new(width, height);

        // BBC red background
        let bbc_red = Rgba([234, 68, 57, 255]);

        // Fill with red
        for pixel in img.pixels_mut() {
            *pixel = bbc_red;
        }

        // Create simple BBC text pattern (simplified blocks)
        // This creates a minimalist "BBC" appearance
        let white = Rgba([255, 255, 255, 255]);

        // Draw simplified "BBC" blocks in center
        let center_x = width / 2;
        let center_y = height / 2;
        let block_width = 15;
        let block_height = 30;
        let spacing = 5;

        // B (left)
        let x1 = center_x - block_width * 2 - spacing * 2;
        draw_rect(&mut img, x1, center_y - block_height / 2, block_width, block_height, white);

        // B (middle)
        let x2 = center_x - block_width - spacing;
        draw_rect(&mut img, x2, center_y - block_height / 2, block_width, block_height, white);

        // C (right)
        let x3 = center_x + spacing;
        draw_rect(&mut img, x3, center_y - block_height / 2, block_width, block_height, white);

        DynamicImage::ImageRgba8(img)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

// Helper to draw a rectangle
fn draw_rect(img: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32, color: Rgba<u8>) {
    for dy in 0..height {
        for dx in 0..width {
            let px = x + dx;
            let py = y + dy;
            if px < img.width() && py < img.height() {
                img.put_pixel(px, py, color);
            }
        }
    }
}

// Global cache instance using lazy_static
lazy_static::lazy_static! {
    pub static ref GLOBAL_IMAGE_CACHE: Arc<Mutex<ImageCache>> = Arc::new(Mutex::new(ImageCache::new()));
}

/// Get image from global cache
pub fn get_image(url: Option<&str>) -> DynamicImage {
    match url {
        Some(url_str) => {
            let mut cache = GLOBAL_IMAGE_CACHE.lock().unwrap();
            cache.get_or_load(url_str)
        }
        None => {
            ImageCache::create_bbc_logo()
        }
    }
}
