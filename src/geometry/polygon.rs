use crate::utils::points::Point;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Size {
    pub x: u32,
    pub y: u32
}
impl Size {
    fn new () -> Self {
        Self {x: 0, y: 0}
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub points: Vec<Point>, 
    pub min: Point,
    pub max: Point,  
    pub size: Size,
    pub position: Point
}

impl Polygon {
    /// Constructs a new polygon with minimum and maximum bounds.
    pub fn new(points: &[Point]) -> Self {

        let empty: bool = points.is_empty();
        let (min, max) = if empty {(0, 0)} else {(i32::MAX, i32::MIN)};

        let mut polygon = Self {
            points: Vec::new(),
            min: Point::new(min, min),
            max: Point::new(max, max),
            size: Size::new(),
            position: Point::new(0, 0),
        };

        if !empty {
            polygon.extend(points);
        }

        polygon

    }

    pub fn insert(&mut self, points: &[Point]) {
        for p in points {
            self.points.insert(0, *p);
        }
        self.update(points);
    }

    pub fn push(&mut self, point: Point) {
        self.points.push(point);
        self.update(&[point]);
    }

    /// Adds points to the polygon and updates the bounds.
    pub fn extend(&mut self, points: &[Point]) {
        self.points.extend(points);
        self.update(points);
    }

    fn update(&mut self, points: &[Point]) {
        for p in points {
            self.update_bounds(p);
        }
        self.update_size();
        self.update_position();
    }

    /// Updates the min/max `x` and `y` values.
    fn update_bounds(&mut self, point: &Point) {
        self.min.y = self.min.y.min(point.y);
        self.max.y = self.max.y.max(point.y);
        self.min.x = self.min.x.min(point.x);
        self.max.x = self.max.x.max(point.x);
    }

    fn update_size(&mut self) {
        self.size = Size {x: self.x_range(), y: self.y_range()};
    }

    /// The position is updated in the ratio `1 size : 2 position` units
    /// <br> X Y Size in Microbricks is 2 X Y Position in Halfs
    fn update_position(&mut self) {
        self.position = Point { 
            x: self.min.x * 2 + self.size.x as i32,
            y: self.min.y * 2 + self.size.y as i32,
        }
    }

    fn x_range(&self) -> u32 {
        if self.max.x < self.min.x {
            eprintln!(
                "Invalid bounds: min.x = {}, max.x = {}",
                self.min.x, self.max.x
            );
        }
        (self.max.x as i32 - self.min.x as i32).abs() as u32
    }
    
    fn y_range(&self) -> u32 {
        (self.max.y as i32 - self.min.y as i32).abs() as u32
    }

    /// Constructs a `Polygon` from tuples of `(i32, i32)` points.
    pub fn from_tuples(tuples: &[(i32, i32)]) -> Self {
        let points: Vec<Point> = tuples.iter().map(|&(x, y)| Point::new(x, y)).collect();
        Self::new(&points)
    }

    pub fn contains(&self, point: &Point) -> bool {
        // Check if the point is outside the bounding box
        if point.x < self.min.x || point.x > self.max.x || point.y < self.min.y || point.y > self.max.y {
            return false;
        }
    
        let mut crossings = 0;
        let n = self.points.len();
    
        for i in 0..n {
            let p1 = &self.points[i];
            let p2 = &self.points[(i + 1) % n];
    
            // Check if the edge crosses the horizontal line at `point.y`
            if (p1.y > point.y) != (p2.y > point.y) {
                let intersection_x = (p2.x - p1.x) * (point.y - p1.y) / (p2.y - p1.y) + p1.x;
                if point.x < intersection_x {
                    crossings += 1;
                }
            }
        }
    
        // A point is inside the polygon if the number of crossings is odd
        crossings % 2 == 1
    }

}