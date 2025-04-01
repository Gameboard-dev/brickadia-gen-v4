use brickadia::save::Brick;
use image::{Rgb, RgbImage};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use std::f32::{self, consts::PI};
use std::sync::{Arc, Mutex};
use crate::draw::colors::{BLACK, RED};
use crate::draw::draw::DebugImage;
use crate::geometry::arc::{Arc as AngleArc, WedgeArc};
use crate::geometry::diagonal::diagonal_as_triangles;
use crate::utils::indicatif::pb;
use crate::utils::points::Point;
use crate::utils::sfc32::{random_range, sfc32};
use super::cell::Cell;

struct ThetaMaze {
    ring_width: f32,
    rings: usize,
    initial_divisions: usize,
    maze: Vec<Vec<Cell>>,
    backtrack_path: Vec<(usize, usize)>,
    canvas_size: u32,
    centre: Point,
}

impl ThetaMaze {
    fn new(ring_width: u32, rings: usize, initial_divisions: usize) -> Self {

        let dimensions = || -> u32 {
            let diameter = ring_width * (rings as u32 * 2);
            let padding = 100;
            diameter + padding
        };

        let maze: Vec<Vec<Cell>> = Vec::new();

        let size = dimensions();
        let mid = size as i32 / 2;

        Self {
            ring_width: ring_width as f32,
            rings,
            initial_divisions,
            maze,
            canvas_size: size,
            centre: Point::new(mid, mid),
            backtrack_path: Vec::new(),
        }
    }

    fn divisions_in_ring(&self, ring: usize) -> usize {
        self.initial_divisions * (2 as usize).pow((ring / 2) as u32)
    }

    fn unvisited_neighbours(&self, ring: usize, division: usize) -> Vec<(usize, usize)> {

        let mut unvisited: Vec<(usize, usize)> = Vec::new();

        let total_divisions: usize = self.divisions_in_ring(ring);

        let odd_ring: bool = ring % 2 == 1;
        let even_ring: bool = !odd_ring;

        // Iterate over the neighbours of the current division within the ring.
        let left_division: usize = (division + total_divisions - 1) % total_divisions;
        let right_division: usize = (division + 1) % total_divisions;

        for &neighbor in [left_division, right_division].iter() {
            if !self.maze[ring][neighbor].visited {
                unvisited.push((ring, neighbor));
            }
        }
        // The inner neighbors are added only if our ring isn't the innermost one:
        if ring > 0 {
            // For an odd-numbered ring, the inner ring's divisions align one-to-one with the outer ring.
            // For an even-numbered ring, each division in the inner ring corresponds to two divisions in the outer ring. 
            // - To find the matching inner division, you perform integer division with 2.
            let inner: usize = 
                if odd_ring { division } else 
                            { division / 2 }; 

            if !self.maze[ring - 1][inner].visited {
                unvisited.push((ring - 1, inner));
            }
        }
        // The outer neighbors are added if the ring isn't the outermost one:
        if ring < self.rings - 1 
        {
            if odd_ring {
                // outer1 is our index * 2
                let outer1: usize = division * 2;
                if !self.maze[ring + 1][outer1].visited {
                    unvisited.push((ring + 1, outer1));
                }
                // outer2 is our index * 2 + 1
                let outer2: usize = outer1 + 1;
                if !self.maze[ring + 1][outer2].visited {
                    unvisited.push((ring + 1, outer2));
                }
                } 
            else if even_ring && !self.maze[ring + 1][division].visited {
                unvisited.push((ring + 1, division));
            }
        }
        unvisited
    }

    fn open_wall_between(&mut self, ringdiv_a: (usize, usize), ringdiv_b: (usize, usize)) {
        if ringdiv_a.0 == ringdiv_b.0 {
            // Same ring, right wall
            let ring = ringdiv_a.0;
            let div1 = ringdiv_a.1;
            let div2 = ringdiv_b.1;
            let next = (div1 + 1) % self.divisions_in_ring(ring);
            if div2 == next {
                self.maze[ring][div1].right_wall = false;
            } else {
                self.maze[ring][div2].right_wall = false;
            }
        } else {
            // Different rings, open inner wall of outer cell
            let (_, outer) = if ringdiv_a.0 < ringdiv_b.0 { (ringdiv_a, ringdiv_b) } else { (ringdiv_b, ringdiv_a) };
            self.maze[outer.0][outer.1].inner_wall = false;
        }
    }

    fn generate(&mut self, seed: (u32, u32, u32, u32)) {

        let (a, b, c, d) = seed;
        let mut rng = sfc32(a, b, c, d);

        for i in 0..self.rings {
            let divisions = self.divisions_in_ring(i);
            self.maze.push(vec![Cell::new(); divisions]);
        }

        let mut ring = self.rings - 1;
        let mut division = 0;

        self.maze[ring][division].visited = true;
        self.maze[ring][division].outer_wall = false;

        for i in 0..self.initial_divisions {
            self.maze[0][i].right_wall = false;
            self.maze[0][i].visited = true;
        }

        let mut backtrack_path = vec![];
        let mut solution_path = vec![];
        let mut centre_entry = 0;

        loop {
            let candidates = self.unvisited_neighbours(ring, division);
            let candidates_len = candidates.len();
            if !candidates.is_empty() {
                backtrack_path.push((ring, division));
                let next = candidates[random_range(&mut rng, 0.0, candidates_len as f32)];
                self.open_wall_between((ring, division), next);
                ring = next.0;
                division = next.1;
                self.maze[ring][division].visited = true;

                if ring == 1 {
                    centre_entry = division;
                    solution_path = backtrack_path.clone();
                    solution_path.push((ring, division));
                    solution_path.push((0, division));
                }
            } else if let Some((r, d)) = backtrack_path.pop() {
                ring = r;
                division = d;
            } else {
                break;
            }
        }
        // Create an entry to the centre on the backtrack path.
        self.maze[1][centre_entry].inner_wall = false;
        self.backtrack_path = solution_path;
    }
    
    fn calculate_rgb(&self, ring: usize, total_rings: usize) -> Rgb<u8> {
        let normal = ring as f32 / total_rings as f32;
        let brightness = ((normal - 0.7) * 255.0) as u8;
        Rgb([brightness, brightness, brightness])
    }   

    fn canvas (&self) -> RgbImage {
        let grey = Rgb([255, 255, 255]);
        RgbImage::from_pixel(self.canvas_size, self.canvas_size, grey)
    }

    pub fn build(&self, draw: bool, solve: bool, mut bricks: &mut Vec<Brick>) {
    
        let mut maze_debug = if draw {DebugImage::new(Some(self.canvas()), BLACK)} 
                        else {DebugImage::new(None, BLACK)};
        
        let mut bricks_debug = maze_debug.clone();

        self.draw_maze(&mut maze_debug, &mut bricks_debug, &mut bricks);
        
        if solve {
            self.draw_solution_path(&mut maze_debug);
        }
        
        maze_debug.save("maze.png");
        bricks_debug.save("maze_bricks.png");
    }

    fn arc_as_bricks(&self, rgb: Rgb<u8>, mut bricks_debug: &mut DebugImage, mut bricks: &mut Vec<Brick>,
                    radius: f32, begin_angle: f32, end_angle: f32) {

        if radius > 0.0 {
            let radius = radius.round() as u32;
            let mut wedge_arc = 
            WedgeArc
            {
                rgb, 
                arc: AngleArc {
                    begin_angle,
                    end_angle,
                    centre: self.centre,
                    is_inner_arc: false,
                    radius,
                }, 
                radius_gap: 50
            };
            //println!("Arc: {:?}", wedge_arc.arc);
            wedge_arc.compute(&mut bricks_debug, &mut bricks);
        }
    }
    
    fn draw_maze(
        &self,
        maze_debug: &mut DebugImage,
        bricks_debug: &mut DebugImage,
        bricks: &mut Vec<Brick>,
    ) {
        let maze_debug = Arc::new(Mutex::new(maze_debug));
        let bricks_debug = Arc::new(Mutex::new(bricks_debug));
        let bricks = Arc::new(Mutex::new(bricks));
        let progress = Arc::new(Mutex::new(pb(self.rings as u64, "Building Maze...", "yellow/orange")));
    
        // If the maze has holes, it's because 2 contiguous arcs are trying to occupy the same space.
        (0..self.rings)
            .into_par_iter()
            .for_each(|ring| {
                let mut maze_dbg = maze_debug.lock().unwrap();
                let mut bricks_dbg = bricks_debug.lock().unwrap();
                let mut bricks_guard = bricks.lock().unwrap();
                let progress_guard = progress.lock().unwrap();
    
                let divisions = self.divisions_in_ring(ring);
                let arc_angle = 2.0 * std::f32::consts::PI / divisions as f32;
                let radius_inner = self.ring_width * ring as f32;
                let radius_outer = radius_inner + self.ring_width;
                let rgb = self.calculate_rgb(ring, self.rings);
                let (cx, cy) = self.centre.tuple();
    
                let mut current_arcs = [(None, radius_inner, rgb), (None, radius_outer, BLACK)];
    
                for division in 0..divisions {
                    let cell = &self.maze[ring][division];
                    let start_angle = arc_angle * division as f32;
                    let end_angle = start_angle + arc_angle;
    
                    // Handle inner and outer arcs
                    for (arc_start, radius, color) in &mut current_arcs {
                        let is_wall = if *radius == radius_inner {
                            cell.inner_wall
                        } else {
                            ring == self.rings - 1 && cell.outer_wall
                        };
    
                        if is_wall {
                            if arc_start.is_none() {
                                *arc_start = Some(start_angle);
                            }
                        } else if let Some(start) = *arc_start {
                            maze_dbg.draw_arc(cx, cy, *radius, start, start_angle, *color);
                            self.arc_as_bricks(*color, &mut bricks_dbg, &mut bricks_guard, *radius, start, start_angle);
                            *arc_start = None;
                        }
                    }
    
                    // Perpendicular intersections
                    if cell.right_wall {
                        let (sx, sy) = (
                            cx + radius_inner * end_angle.cos(),
                            cy + radius_inner * end_angle.sin(),
                        );
                        let (ex, ey) = (
                            cx + radius_outer * end_angle.cos(),
                            cy + radius_outer * end_angle.sin(),
                        );
                        let (from, to) = ((sx, sy), (ex, ey));
                        maze_dbg.draw_line(from, to, rgb);
                        bricks_dbg.draw_line(from, to, RED);

                        /* WORK IN PROGRESS
                        
                        // Needs to distinguish between diagonal and straightline approximation
                        // Lines which are close to straightline are stepped rectangles etc.
                        
                        for triangle in diagonal_as_triangles(Point::from_f32(from), Point::from_f32(to)) {
                            bricks_dbg.draw_outline(&triangle, rgb);
                        };

                        */

                    }
                }
    
                // Finalize any remaining arcs
                for (arc_start, radius, color) in &mut current_arcs {
                    if let Some(start) = *arc_start {
                        maze_dbg.draw_arc(cx, cy, *radius, start, arc_angle * divisions as f32, *color);
                        self.arc_as_bricks(*color, &mut bricks_dbg, &mut bricks_guard, *radius, start, arc_angle * divisions as f32);
                    }
                }
    
                progress_guard.inc(1);
            });
    }
    
    fn draw_solution_path(&self, maze_debug: &mut DebugImage) {

        if self.backtrack_path.is_empty() {
            return;
        }

        let mut end: Option<(f32, f32)> = None;
        let (cx, cy): (f32, f32) = self.centre.tuple();
    
        for &(ring, division) in &self.backtrack_path {
            let divisions = self.divisions_in_ring(ring);
            let arc_angle = 2.0 * PI / divisions as f32;
            let mid_angle = arc_angle * division as f32 + arc_angle / 2.0;
            let radius = self.ring_width * (ring as f32 + 0.5);
            let p = (
                cx + radius * mid_angle.cos(),
                cy + radius * mid_angle.sin(),
            );
    
            if let Some(prev) = end {
                maze_debug.draw_line(prev, p, RED);
            }
            end = Some(p);
        }
    }
}



#[cfg(test)]
mod generate_maze {

    use std::env;
    use crate::metadata::save::save_bricks;
    use super::*;
    
    #[test]
    fn run() {

        let rings = 3;
        let ring_width = 100;
        let initial_divisions = 2;
    
        let mut maze = ThetaMaze::new(ring_width, rings, initial_divisions);
        let seed = (11, 12, 15, 2);
        // The same seed will generate the SAME maze.
        maze.generate(seed);
    
        let draw = true; // computes a canvas based on the dimensions and draws a debug image of the maze.
        let solve = false; // solves the maze with a pathfinding algorithm used to construct it.
    
        let mut bricks: Vec<Brick> = Vec::new();
        maze.build(draw, solve, &mut bricks);

        unsafe { env::set_var("NAME", "Maze"); }

        save_bricks(bricks);
    }

}
