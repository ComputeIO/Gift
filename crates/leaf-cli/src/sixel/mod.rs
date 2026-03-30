//! Direct Sixel rendering for SVG diagrams.
//!
//! This module provides pure Rust Sixel image rendering without external dependencies.

use image::{ImageBuffer, Rgba};

/// Renders an RGBA image buffer as Sixel format.
pub fn encode_sixel(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
    let (img_width, img_height) = image.dimensions();
    let mut output = String::new();

    // Sixel header: DECSIXEL
    output.push_str("\x1bP0;1;0q");

    // Define palette: color 1 = white, color 2 = black
    // Format: #palette;2;R;G;B (2 = SGR mode)
    output.push_str("#1;2;255;255;255"); // White
    output.push_str("#2;2;0;0;0"); // Black

    // Raster attributes: ;pan;pads;height;width
    output.push_str(&format!(";1;1;{};{}", img_height, img_width));

    // Select color 2 (black foreground)
    output.push_str("#2");

    let band_height = 6;
    let bands = (img_height + band_height - 1) / band_height;

    for band_y in 0..bands {
        let y_start = band_y * band_height;
        let y_end = std::cmp::min(y_start + band_height, img_height);
        let actual_height = (y_end - y_start) as usize;

        // Encode each column
        for x in 0..img_width {
            let mut sixel_bits: u8 = 0;
            for (row_idx, y) in (y_start..y_end).enumerate() {
                let luma = luminance(image.get_pixel(x, y));
                // If pixel is dark (foreground), set corresponding bit
                if luma < 128 {
                    sixel_bits |= 1 << (actual_height - 1 - row_idx);
                }
            }
            if sixel_bits != 0 {
                if let Some(c) = char::from_u32(sixel_bits as u32 + 63) {
                    output.push(c);
                }
            }
        }

        // End band
        output.push('"');
    }

    // End Sixel (DECOSC)
    output.push_str("\x1b\\");

    output
}

/// Calculates luminance for a pixel (0-255 range).
fn luminance(pixel: &Rgba<u8>) -> u8 {
    let r = pixel[0] as f32;
    let g = pixel[1] as f32;
    let b = pixel[2] as f32;

    // ITU-R BT.709 luminance coefficients
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    luma as u8
}

/// Renders SVG string to Sixel output.
pub fn render_svg_to_sixel(svg: &str, width: u32, height: u32) -> Result<String, String> {
    // Parse SVG using resvg
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg, &opt)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Create pixel buffer
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Render SVG to pixmap
    let scale_x = width as f32 / tree.size().width();
    let scale_y = height as f32 / tree.size().height();
    let transform = resvg::tiny_skia::Transform::from_scale(scale_x, scale_y);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to image buffer
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, pixmap.take()).unwrap();

    // Encode as Sixel
    Ok(encode_sixel(&img))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sixel_header() {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(10, 10);
        let sixel = encode_sixel(&img);
        assert!(sixel.starts_with("\x1bP0;1;0q"));
        assert!(sixel.ends_with("\x1b\\"));
    }

    #[test]
    fn test_render_svg_simple() {
        let svg = r#"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect x="10" y="10" width="80" height="80" fill="red"/>
        </svg>"#;

        let result = render_svg_to_sixel(svg, 100, 100);
        assert!(result.is_ok());
        let sixel = result.unwrap();
        assert!(sixel.starts_with("\x1bP0;1;0q"));
    }
}
