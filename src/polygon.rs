use rand::Rng;

use crate::primitives::{DirEdge, Point, PointOrientation};

#[derive(Debug)]
pub struct SimplePolygon {
    point_list: Vec<Point>, //Circular list of points
}

impl SimplePolygon {
    pub fn get_point_list(&self) -> &Vec<Point> {
        &self.point_list
    }

    pub fn partition_monotone(&self) {
        let point_list = &self.point_list;
        let get_next_index = |point_index: usize| (point_index + 1) % point_list.len();
        let get_prev_index = |point_index: usize| {
            if point_index == 0 {
                return point_list.len() - 1;
            }
            point_index - 1
        };
        struct EdgePoints<'a> {
            higher: &'a Point,
            lower: &'a Point,
        }
        let get_edgepoints = |point_index| {
            let p1 = &point_list[point_index];
            let p2 = &point_list[get_next_index(point_index)];
            if p1.is_higher_than(p2) {
                return EdgePoints {
                    higher: p1,
                    lower: p2,
                };
            } else {
                return EdgePoints {
                    higher: p2,
                    lower: p1,
                };
            }
        };

        enum PointType {
            Split,
            Merge,
            Start,
            End,
            Regular,
        }

        let is_reflex = |point_index| {
            let prev = &point_list[get_prev_index(point_index)];
            let curr = &point_list[point_index];
            let next = &point_list[get_next_index(point_index)];

            //SimplePolygon has Counterclockwise orientation be default
            if Point::orientation(prev, curr, next) == PointOrientation::Clockwise {
                return true;
            }
            false
        };

        let mut event_queue: Vec<usize> = (0..point_list.len()).collect();
        event_queue.sort_by(|a, b| {
            if point_list[*a].is_higher_than(&point_list[*b]) {
                return std::cmp::Ordering::Greater;
            }
            return std::cmp::Ordering::Less;
        });
        event_queue.reverse();

        //        let sweep_line_status = Vec::new();
        for point in event_queue {
            dbg!(point, is_reflex(point));
        }
    }

    fn get_determinant(&self) -> isize {
        let mut ret = 0;
        let len = self.point_list.len();
        for i in 0..len {
            let p_curr = &self.point_list[i];
            let p_next = &self.point_list[(i + 1) % len];
            let val = p_curr.x * p_next.y - p_curr.y * p_next.x;
            ret += val;
        }
        ret
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
            //Checking if points are collinear. We don't want to deal with this.
            //This can be done much much better
            for i in 0..vertex_count - 2 {
                use crate::primitives::*;
                let p0 = &point_list[i + 0];
                let p1 = &point_list[i + 1];
                let p2 = &point_list[i + 2];
                if let PointOrientation::Collinear = Point::orientation(p0, p1, p2) {
                    return None;
                }
            }

            Some(SimplePolygon { point_list })
        };
        for _ in 1..retry_cnt {
            if let Some(mut x) = engine() {
                let d = x.get_determinant();
                if d < 0 {
                    x.point_list.reverse()
                };
                return Some(x);
            }
        }
        None
    }
}
#[cfg(test)]
mod polygon_tests {
    use super::*;
    #[test]
    fn test_area() {
        let p = SimplePolygon::gen_rand_hard(3, 5, 100).unwrap();
        println!("{:?} -- {}", p, p.get_determinant());
    }
}
