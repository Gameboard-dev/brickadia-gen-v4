use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;

use crate::utils::points::Point;


#[derive(Clone)]
pub struct DebugImage {
    img: Option<RgbImage>,
    pub rgb: Rgb<u8>
}

impl DebugImage {
    pub fn new(img: Option<RgbImage>, rgb: Rgb<u8>) -> Self {
        Self { img, rgb }
    }

    /// If an image exists, perform an action on it.
    fn with_image<F>(&mut self, action: F)
    where
        F: FnOnce(&mut RgbImage),
    {
        if let Some(ref mut img) = self.img {
            action(img);
        }
    }

    pub fn is_active(&self) -> bool {
        self.img.is_some()
    }

    pub fn draw_arc(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: Rgb<u8>,
    ) {
        self.with_image(|img| draw_arc(img, center_x, center_y, radius, start_angle, end_angle, color));
    }

    pub fn draw_point(&mut self, p: Point, color: Rgb<u8>) {
        self.with_image(|img| draw_point(img, p, color));
    }

    pub fn draw_line(
        &mut self,
        begin: (f32, f32),
        end: (f32, f32),
        color: Rgb<u8>,
    ) {
        self.with_image(|img| draw_line_segment_mut(img, begin, end, color));
    
    }

    pub fn draw_line_series(&mut self, points: &[Point], color: Rgb<u8>) {
        if points.len() < 2 {
            return; // or handle the error as needed
        }
        self.with_image(|img| {
            for pair in points.windows(2) {
                draw_line_segment_mut(img, pair[0].tuple(), pair[1].tuple(), color);
            }
        });
    }

    pub fn draw_outline(&mut self, polygon: &[Point], color: Rgb<u8>) {
        self.with_image(|img| draw_outline(img, polygon, color));
    }

    pub fn draw_filled_polygon(&mut self, polygon: &[Point], color: Rgb<u8>) {
        self.with_image(|img| draw_filled_polygon(img, polygon, color));
    }

    pub fn save(&self, path: &str) {
        if let Some(ref img) = self.img {
            img.save(path).expect("Failed to save image");
        }
    }
}

/// Clamps a point to valid image coordinates and returns as f32.
fn clamp(img: &RgbImage, p: Point) -> (f32, f32) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    (p.x.clamp(0, w - 1) as f32, p.y.clamp(0, h - 1) as f32)
}

/// Draw a single point on the image.
fn draw_point(img: &mut RgbImage, p: Point, color: Rgb<u8>) {
    let (x, y) = clamp(img, p);
    img.put_pixel(x as u32, y as u32, color);
}

/// Draws an outline given by a loop of points.
pub fn draw_outline(img: &mut RgbImage, polygon: &[Point], color: Rgb<u8>) {
    if polygon.len() < 2 {
        return;
    }
    // Iterate over consecutive pairs, wrapping the last to the first.
    for (a, b) in polygon.iter().zip(polygon.iter().cycle().skip(1).take(polygon.len())) {
        let (sx, sy) = clamp(img, *a);
        let (ex, ey) = clamp(img, *b);
        draw_line_segment_mut(img, (sx, sy), (ex, ey), color);
    }
}

/// Clears the image by setting every pixel to white.
pub fn clear_image(img: &mut RgbImage) {
    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }
}

/// Approximates an arc with sequential line segments.
pub fn draw_arc(
    image: &mut RgbImage, 
    center_x: f32, 
    center_y: f32,
    radius: f32,
    start_angle: f32, 
    end_angle: f32,
    color: Rgb<u8>,
) {
    let steps = 100;
    let angle_step = (end_angle - start_angle) / steps as f32;
    let mut prev_x = center_x + radius * start_angle.cos();
    let mut prev_y = center_y + radius * start_angle.sin();

    for i in 1..=steps {
        let angle = start_angle + i as f32 * angle_step;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        draw_line_segment_mut(image, (prev_x, prev_y), (x, y), color);
        prev_x = x;
        prev_y = y;
    }
}


/// Draws a filled polygon given by a loop of points.
pub fn draw_filled_polygon(img: &mut RgbImage, polygon: &[Point], color: Rgb<u8>) {
    if polygon.len() < 3 {
        return; // A polygon must have at least 3 points to be filled.
    }

    // Find the bounding box of the polygon.
    let min_y = polygon.iter().map(|p| p.y).min().unwrap_or(0);
    let max_y = polygon.iter().map(|p| p.y).max().unwrap_or(0);

    // Iterate over each scanline (y-coordinate) within the bounding box.
    for y in min_y..=max_y {
        // Find intersections of the polygon edges with the current scanline.
        let mut intersections = Vec::new();
        for (a, b) in polygon.iter().zip(polygon.iter().cycle().skip(1).take(polygon.len())) {
            if (a.y <= y && b.y > y) || (b.y <= y && a.y > y) {
                // Calculate the x-coordinate of the intersection.
                let t = (y - a.y) as f64 / (b.y - a.y) as f64;
                let x = a.x as f64 + t * (b.x - a.x) as f64;
                intersections.push(x as i32);
            }
        }

        // Sort the intersections by x-coordinate.
        intersections.sort_unstable();

        // Fill the pixels between pairs of intersections.
        for pair in intersections.chunks(2) {
            if let [start, end] = pair {
                for x in *start..=*end {
                    let (sx, sy) = clamp(img, Point { x, y });
                        img.put_pixel(sx as u32, sy as u32, color);
                    }
                }
            }
        }
    }