
use std::{cmp::max, f32::consts::PI};
use brickadia::save::{Brick, Direction, Rotation, Size};
use crate::draw::colors::rgb_to_brick;
use crate::draw::draw::DebugImage;
use crate::metadata::assets::BrickAssets;
use crate::utils::points::Point;
use image::Rgb;
use super::polygon::Polygon;

use super::decompose::render_as_squares;

pub const SEMICIRCLE: f32 = PI;
pub const CIRCLE: f32 = PI * 2.0;

#[derive(Clone, Debug)]
pub struct Arc {
    pub(crate)begin_angle: f32,
    pub(crate)end_angle: f32,
    /// Higher values lead to less precise approximations.
    pub(crate)centre: Point,
    pub(crate)radius: u32,
    pub(crate)is_inner_arc: bool,
}
impl Arc {

    fn coordinate (&self, angle: f32) -> Point {
        Point { 
            x: (self.centre.x as f32 + self.radius as f32 * angle.cos()).round() as i32,
            y: (self.centre.y as f32 + self.radius as f32 * angle.sin()).round() as i32, 
        }
    }

    pub fn in_even_quadrants(&self, p: &Point) -> bool {
        (p.x >= self.centre.x) == (p.y >= self.centre.y)
    }
    
    fn vertex_angles(&self, angle_step: f32, steps: usize) -> impl DoubleEndedIterator<Item = (f32, f32, f32)> {
        (0..steps).map(move |i| {
            let angle1 = self.begin_angle + i as f32 * angle_step;
            let angle2 = angle1 + angle_step;
            let mid = (angle1 + angle2) / 2.0;
            (angle1, mid, angle2)
        })
    }

    pub fn vertex_points(&self) -> impl DoubleEndedIterator<Item = [Point; 3]> {

        let p = |angle| self.coordinate(angle);

        let angle_span = self.end_angle - self.begin_angle;
        let raw_steps = (self.radius as f32 * angle_span).ceil() as usize;
        
        let steps = max(raw_steps / 30, 1);

        let angle_step = angle_span / steps as f32;

        //println!("Steps: {}, {}", raw_steps, steps);

        self.vertex_angles(angle_step, steps)
            .map(move |(angle1, mid, angle2)| {

                let (p1, mid, p3) = (p(angle1), p(mid), p(angle2));

                // The coordinates of the 90deg vertex
                let p90 = if self.in_even_quadrants(&mid) == self.is_inner_arc {
                    Point::new(p1.x, p3.y)
                } else {
                    Point::new(p3.x, p1.y)
                };

                [p(angle1), p90, p(angle2)]
            }).into_iter()
    }
    pub fn concentric(&mut self, radius: u32) -> Arc {
        Arc {
            radius,
            is_inner_arc: radius < self.radius,
            ..self.clone()
        }
    }

}


pub struct Wedge {
    brick: Brick,
    points: [Point; 3]
}

pub struct WedgeArc {
    pub rgb: Rgb<u8>,
    pub arc: Arc,
    pub radius_gap: u32,
}

impl WedgeArc {

    pub fn end_arc_corners(
        &mut self,
        inner_wedges: &Vec<Polygon>,
        outer_wedges: &Vec<Polygon>,
    ) -> [Point; 2] {
        let in_even_quadrants = |p: &Point| self.arc.in_even_quadrants(p);
    
        let ends = |wedges: &Vec<Polygon>| {
            (
                wedges.first().unwrap().points.clone(),
                wedges.last().unwrap().points.clone(),
            )
        };
    
        let (inner_beginning, inner_ending) = ends(inner_wedges);
        let (outer_beginning, outer_ending) = ends(outer_wedges);
    
        // Closure to compute the end vertex
        let corner_vertex = |inner: &Point, outer: &Point, inverse: bool| -> Point {
            let options = [[outer.x, inner.y], [inner.x, outer.y]];
            let arc_end_vertex = options[(in_even_quadrants(outer) ^ inverse) as usize];
            Point::from_iter(arc_end_vertex)
        };
    
        // Compute the two vertices
        let corner_begin = corner_vertex(&inner_beginning[0], &outer_beginning[0], true);
        //debug.draw_line_series(&corner_begin, BLUE);

        let corner_end = corner_vertex(&inner_ending[2], &outer_ending[2], false);
        //debug.draw_line_series(&corner_end, RED);
    
        [corner_begin, corner_end]
    }

    fn wedge_brick(&self, wedge: &Polygon) -> Option<Brick> {
        let (size, position) = (wedge.size, wedge.position);
        let p90 = wedge.points[1];
        if size.x != 0 && size.y != 0 {
            Some(Brick {
                asset_name_index: BrickAssets::MicroWedge.index() as u32,
                color: rgb_to_brick(self.rgb),
                size: Size::Procedural(size.x, size.y, 100),
                position: (position.x, position.y, 100),
                rotation: if p90.y == wedge.max.y { Rotation::Deg180 } else { Rotation::Deg0 },
                direction: if p90 == wedge.min || p90 == wedge.max { Direction::ZPositive } else { Direction::ZNegative },
                ..Default::default()
            })
        } else {
            None
        }
    }

    fn rectangle_brick(&self, rectangle: &Polygon) -> Brick {
        let (size, position) = (rectangle.size, rectangle.position);
        Brick {
                asset_name_index: BrickAssets::MicroBrick.index() as u32,
                color: rgb_to_brick(self.rgb),
                size: Size::Procedural(size.x, size.y, 100),
                material_index: 0,
                position: (position.x, position.y, 100),
                ..Default::default()
        }
    }

    fn build_wedges_and_rectangles(
        &mut self,
        inner_arc: Arc,
        outer_arc: Arc,
        bricks: &mut Vec<Brick>,
        debug: &mut DebugImage,
    ) {
    
        let mut inner_wedges: Vec<Polygon> = Vec::new();
        let mut outer_wedges: Vec<Polygon> = Vec::new();
        let mut inner_points: Vec<Point> = Vec::new();
        let mut outer_points: Vec<Point> = Vec::new();
        let mut wedge_bricks: Vec<Brick> = Vec::new();
        
        for (arc, (wedges, vertices)) in [inner_arc, outer_arc]
            .iter()
            .zip([&mut inner_wedges, &mut outer_wedges].iter_mut().zip([&mut inner_points, &mut outer_points].iter_mut()))
        {
            for vertex_group in arc.vertex_points() {
                let wedge = Polygon::new(&vertex_group);
                if let Some(brick) = self.wedge_brick(&wedge) {
                    wedge_bricks.push(brick);
                    debug.draw_outline(&vertex_group, self.rgb);
                }
                wedges.push(wedge);
                vertices.extend(vertex_group);
            }
        }

        let [corner_end, corner_begin] = self.end_arc_corners(&inner_wedges, &outer_wedges);

        let mut polygon = Polygon::new(&[]);
        polygon.push(corner_end);
        polygon.extend(&inner_points);
        polygon.push(corner_begin);

        outer_points.reverse();
        polygon.extend(&outer_points);


        for rectangle in render_as_squares(polygon, debug) {
            bricks.push(self.rectangle_brick(&rectangle)) 
        }

        for wedge in wedge_bricks {
            bricks.push(wedge);
        }

    }

    /// Processes an arc into wedges and draws corner lines.
    pub fn compute(&mut self, debug: &mut DebugImage, mut bricks: &mut Vec<Brick>) {

        let mut outer_arc = self.arc.clone();
        let inner_radius = outer_arc.radius.saturating_sub(self.radius_gap);
        let inner_arc = outer_arc.concentric(inner_radius);

        // Wedge vertices used to calculate the arc corners.
        self.build_wedges_and_rectangles(inner_arc, outer_arc, &mut bricks, debug);
        
    }
}

#[cfg(test)]
mod render_arc {

    use super::*;
    use std::env;
    use image::RgbImage;
    use crate::{draw::colors::{BLACK, WHITE}, metadata::save::save_bricks};

    #[test]
    fn run() {

        let mut debug = DebugImage::new(Some(RgbImage::from_pixel(2000, 2000, WHITE)), BLACK);

        let angle_span = rand::random_range(0.5..2.0 * PI);

        let (begin_angle, end_angle) = (0.0, angle_span);

        let mut wedge_arc = 
        WedgeArc
        {
            rgb: BLACK, 
            arc: Arc {
                begin_angle,
                end_angle,
                centre: Point::new(500, 500),
                is_inner_arc: false,
                radius: 650,
            }, 
            radius_gap: 50
        };

        let mut bricks: Vec<Brick> = Vec::new();

        wedge_arc.compute(&mut debug, &mut bricks);

        unsafe { env::set_var("NAME", "Arc"); }

        save_bricks(bricks);

        debug.save("circle.png");

    }
}