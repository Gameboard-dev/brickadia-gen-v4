
use crate::draw::draw::DebugImage;
use crate::utils::points::Point;
use super::polygon::Polygon;
use rayon::prelude::*;

pub fn render_as_squares(polygon: Polygon, debug: &mut DebugImage) -> Vec<Polygon> {

    //println!("Polygon {:?}", polygon.points);

    let mut rectangles = Vec::new();

    // Determine the bounding box of the polygon
    let min_x = polygon.min.x;
    let min_y = polygon.min.y;

    // Create a grid to track which points are inside the polygon
    let column_count = (polygon.size.x + 1) as usize;
    let row_count = (polygon.size.y + 1) as usize;
    let mut grid = vec![vec![false; column_count]; row_count];

    // Mark grid cells that are inside the polygon
    grid.par_iter_mut().enumerate().for_each(|(row, row_data)| {
        let y_coord = min_y + row as i32;
        for column in 0..column_count {
            let x_coord = min_x + column as i32;
            let point = Point { x: x_coord, y: y_coord };
            if polygon.contains(&point) {
                row_data[column] = true;
            }
        }
    });

    // Track processed cells to avoid duplicates
    let mut processed = vec![vec![false; column_count]; row_count];

    // Generate merged rectangles
    for row in 0..row_count {
        for column in 0..column_count {
            // Skip cells outside the polygon or already processed
            if grid[row][column] && !processed[row][column] {

                // Rectangle
                let mut width = 0;
                let mut height = 0;

                // Expand horizontally (x-axis)
                while column + width < column_count && grid[row][column + width] && !processed[row][column + width] {
                    width += 1;
                }

                // Expand vertically (y-axis) while maintaining the horizontal width
                'vertical: while row + height < row_count {
                    for dx in 0..width {
                        if !grid[row + height][column + dx] || processed[row + height][column + dx] {
                            // Expansion stops as soon as any column in the row fails the condition.
                            break 'vertical;
                        }
                    }
                    height += 1;
                }

                // Mark the rectangle as processed during expansion
                for dy in 0..height {
                    for dx in 0..width {
                        processed[row + dy][column + dx] = true;
                    }
                }

                // Position and Dimensions
                let (x, y) = (min_x + column as i32, min_y + row as i32);
                let (w, h) = (width as i32, height as i32);

                let tl = Point { x, y };
                let br = Point { x: x + w, y: y + h };
                let tr = Point { x: x + w, y };
                let bl = Point { x, y: y + h };

                let rectangle = Polygon::new(&[tl, tr, br, bl]);
                
                const MAX_LENGTH: i32 = 1000;
                if width as i32 >= MAX_LENGTH || height as i32 >= MAX_LENGTH {
                    let mut divided_rectangles = halve_rectangle(&rectangle, MAX_LENGTH);
                    rectangles.append(&mut divided_rectangles);
                } else {
                    rectangles.push(rectangle);
                }
            }
        }
    }

    // Draw rectangles on the debug image
    for rect in &rectangles {
        debug.draw_filled_polygon(&rect.points, debug.rgb);
        //debug.draw_outline(&rect.points, RED);
        //println!("Rectangle {:?}", rect.points);
    }
    rectangles
}

/// Splits a large rectangle into smaller rectangles if its width or height exceeds `max_length`.
fn halve_rectangle(rectangle: &Polygon, max_size: i32) -> Vec<Polygon> {

    let mut rectangle_halfs = Vec::new();

    let tl = rectangle.points[0];
    let br = rectangle.points[2];

    let mut width = br.x - tl.x;
    let mut height = br.y - tl.y;

    if width >= max_size {
        // Horizontal Splits
        width = width / 2;
        let mid_x = tl.x + width;

        rectangle_halfs.push(Polygon::new(&[
            tl,
            Point { x: mid_x, y: tl.y },
            Point { x: mid_x, y: br.y },
            Point { x: tl.x, y: br.y },
        ]));

        rectangle_halfs.push(Polygon::new(&[
            Point { x: mid_x, y: tl.y },
            Point { x: br.x, y: tl.y },
            br,
            Point { x: mid_x, y: br.y },
        ]));

    } else if height >= max_size {

        height = height / 2;
        let mid_y = tl.y + height;

        rectangle_halfs.push(Polygon::new(&[
            tl,
            Point { x: br.x, y: tl.y },
            Point { x: br.x, y: mid_y },
            Point { x: tl.x, y: mid_y },
        ]));

        rectangle_halfs.push(Polygon::new(&[
            Point { x: tl.x, y: mid_y },
            Point { x: br.x, y: mid_y },
            br,
            Point { x: tl.x, y: br.y },
        ]));

    } else {
        // No splits
        rectangle_halfs.push(rectangle.clone());
    }

    if width >= max_size || height >= max_size {
        return rectangle_halfs.into_iter()
            .flat_map(|r| halve_rectangle(&r, max_size))
            .collect();
    }

    rectangle_halfs
}