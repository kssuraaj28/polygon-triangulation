mod primitives;
use rand::Rng;

use primitives::{DirEdge, Point};

#[derive(Debug)]
pub struct SimplePolygon {
    point_list: Vec<Point>, //Circular list of points
}

impl SimplePolygon {
    pub fn get_point_list(&self) -> &Vec<Point> {
        &self.point_list
    }

    pub fn gen_rand_hard(vertex_count: usize, max_coord: usize, retry_cnt: usize) -> Option<Self> {
        if vertex_count < 3 {
            return None;
        }
        let engine = || {
            let mut point_list: Vec<Point> = Vec::new();

            let mut rng = rand::thread_rng();
            let mut gen_rand_point = || {
                let rand_x = rng.gen_range(0..max_coord);
                let rand_y = rng.gen_range(0..max_coord);
                Point::new(rand_x as isize, rand_y as isize)
            };

            //Generating the first edge
            let p1: Point = gen_rand_point();
            let p2: Point = loop {
                let p2 = gen_rand_point();
                if p2 != p1 {
                    break p2;
                }
            };
            point_list.push(p1);
            point_list.push(p2);
            for idx in 2..vertex_count {
                let p: Point = 'outer: loop {
                    let p = gen_rand_point();
                    if p == point_list[idx - 2] {
                        continue 'outer;
                    }
                    let leading_edge = DirEdge::from_points(&p, point_list.last().unwrap());
                    let slc = &point_list[0..point_list.len() - 1];
                    for (p, q) in slc.iter().zip(slc.iter().skip(1)) {
                        let curr_edge = DirEdge::from_points(p, q);
                        if leading_edge.intersects(&curr_edge) {
                            continue 'outer;
                        }
                    }
                    break p;
                };
                point_list.push(p);
            }

            let final_edge =
                DirEdge::from_points(point_list.first().unwrap(), point_list.last().unwrap());

            let slc = &point_list[1..point_list.len() - 1];
            for (p, q) in slc.iter().zip(slc.iter().skip(1)) {
                let curr_edge = DirEdge::from_points(p, q);
                if final_edge.intersects(&curr_edge) {
                    return None;
                }
            }

            Some(SimplePolygon { point_list })
        };
        for _ in 1..retry_cnt {
            if let Some(x) = engine() {
                return Some(x);
            }
        }
        None
    }
}
