use crate::utils::points::Point;

/// Approximates a diagonal line with right-angled triangles above and below the line.
pub fn diagonal_as_triangles(from: Point, to: Point) -> Vec<Vec<Point>> {
    let mut triangles = Vec::new();
    let (mut x, mut y) = (from.x, from.y);
    let (dx, dy) = ((to.x - from.x).abs(), (to.y - from.y).abs());
    let (step_x, step_y) = (if to.x > from.x { 1 } else { -1 }, if to.y > from.y { 1 } else { -1 });
    let (mut error, primary, secondary) = if dx > dy { (0, dx, dy) } else { (0, dy, dx) };

    while (x, y) != (to.x, to.y) {
        let next_x = x + if dx > dy { step_x } else { 0 };
        let next_y = y + if dx > dy { 0 } else { step_y };

        triangles.push(vec![Point { x, y }, Point { x: next_x, y: y + step_y }, Point { x: next_x, y: next_y }]);
        triangles.push(vec![Point { x, y }, Point { x: x + step_x, y }, Point { x: next_x, y: y + step_y }]);

        error += secondary;
        if error >= primary {
            if dx > dy { /* no-op */ };
            error -= primary;
        }
        x = next_x;
        y = next_y;
    }

    triangles
}