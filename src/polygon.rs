use rand::Rng;
use std::collections::HashMap;

use crate::primitives::{DirEdge, Point, PointOrientation};

#[derive(Debug)]
struct UnorderedEdge<'a> {
    higher: &'a Point,
    lower: &'a Point,
}

#[derive(Debug)]
struct TrapezoidalizationRecord<'a> {
    left_edge: (usize, UnorderedEdge<'a>),
    right_edge: (usize, UnorderedEdge<'a>),
    top_vertex: (usize, &'a Point),
    bottom_vertex: (usize, &'a Point),
}
#[derive(Debug)]
pub struct Trapezoidalization<'a>(Vec<TrapezoidalizationRecord<'a>>);

#[derive(Debug)]
pub struct SimplePolygon {
    point_list: Vec<Point>, //Circular list of points
}

#[derive(Debug, PartialEq)]
enum PointType {
    Split,
    Merge,
    Start,
    End,
    Regular,
}

impl SimplePolygon {
    pub fn get_point_list(&self) -> &Vec<Point> {
        &self.point_list
    }

    fn get_next_index(&self, curr_idx: usize) -> usize {
        (curr_idx + 1) % self.point_list.len()
    }

    fn get_prev_index(&self, curr_idx: usize) -> usize {
        if curr_idx == 0 {
            return self.point_list.len() - 1;
        }
        curr_idx - 1
    }
    fn get_point_type(&self, curr_idx: usize) -> PointType {
        let point_list = &self.point_list;

        let is_reflex = |point_index| {
            let prev = &point_list[self.get_prev_index(point_index)];
            let curr = &point_list[point_index];
            let next = &point_list[self.get_next_index(point_index)];

            //SimplePolygon has Counterclockwise orientation be default
            if Point::orientation(prev, curr, next) == PointOrientation::Clockwise {
                return true;
            }
            false
        };

        let curr = &self.point_list[curr_idx];
        let next_idx = self.get_next_index(curr_idx);
        let next = &self.point_list[next_idx];
        let prev_idx = self.get_prev_index(curr_idx);
        let prev = &self.point_list[prev_idx];

        let rflx = is_reflex(curr_idx);

        if curr.is_higher_than(next) && curr.is_higher_than(prev) {
            if rflx {
                PointType::Split
            } else {
                PointType::Start
            }
        } else if next.is_higher_than(curr) && prev.is_higher_than(curr) {
            if rflx {
                PointType::Merge
            } else {
                PointType::End
            }
        } else {
            PointType::Regular
        }
    }

    pub fn partition_monotone(&self) -> Vec<DirEdge> {
        let mut ret = Vec::new();
        let traps = self.partition_trapezoid();
        for t in traps.0 {
            if self.get_point_type(t.top_vertex.0) == PointType::Merge {
                ret.push(DirEdge::from_points(t.top_vertex.1, t.bottom_vertex.1));
            } else if self.get_point_type(t.bottom_vertex.0) == PointType::Split {
                ret.push(DirEdge::from_points(t.top_vertex.1, t.bottom_vertex.1));
            }
        }
        ret
    }

    pub fn partition_trapezoid(&self) -> Trapezoidalization {
        let point_list = &self.point_list;
        let get_edgepoints = |point_index| {
            let p1 = &point_list[point_index];
            let p2 = &point_list[self.get_next_index(point_index)];
            if p1.is_higher_than(p2) {
                return UnorderedEdge {
                    higher: p1,
                    lower: p2,
                };
            } else {
                return UnorderedEdge {
                    higher: p2,
                    lower: p1,
                };
            }
        };

        //Is point on the left of the line
        fn on_left(e: &UnorderedEdge, p: &Point) -> bool {
            let hp = e.higher;
            let lp = e.lower;
            match Point::orientation(hp, lp, p) {
                PointOrientation::Counterclockwise => false,
                PointOrientation::Clockwise => true,
                PointOrientation::Collinear => {
                    panic!("Can't handle this");
                }
            }
        }

        let mut sweep_line_status: Vec<(usize, usize)> = Vec::new(); //Retard stupid roy

        let mut event_queue: Vec<usize> = (0..point_list.len()).collect();
        event_queue.sort_by(|a, b| {
            if point_list[*a].is_higher_than(&point_list[*b]) {
                return std::cmp::Ordering::Greater;
            }
            return std::cmp::Ordering::Less;
        });
        event_queue.reverse();

        #[derive(Debug)]
        struct TrapezoidPts {
            higher_idx: Option<usize>,
            lower_idx: Option<usize>,
        }

        let mut trapezoids_temp = HashMap::<(usize, usize), TrapezoidPts>::new();

        let mut update_trapezoids = |left_idx, right_idx, point_idx, is_lower| {
            let key = (left_idx, right_idx);
            let entry = trapezoids_temp.entry(key).or_insert(TrapezoidPts {
                higher_idx: None,
                lower_idx: None,
            });
            if is_lower {
                assert_eq!(entry.lower_idx, None);
                entry.lower_idx = Some(point_idx)
            } else {
                assert_eq!(entry.higher_idx, None);
                entry.higher_idx = Some(point_idx)
            }
        };

        for curr_idx in event_queue {
            //            println!("------------------");
            //            println!("{:?} {}", sweep_line_status, curr_idx);
            //            println!("{:?} {:?}", point_list[curr_idx], point_types[&curr_idx]);
            let curr = &point_list[curr_idx];
            //            let next_idx = get_next_index(curr_idx);
            //            let next = &point_list[next_idx];
            let prev_idx = self.get_prev_index(curr_idx);
            //           let prev = &point_list[prev_idx];
            //

            match self.get_point_type(curr_idx) {
                PointType::Start => {
                    let entry_index = if sweep_line_status.is_empty() {
                        0
                    } else {
                        let mut r = sweep_line_status.len();
                        #[cfg(debug_assertions)]
                        for sl_idx in 0..sweep_line_status.len() {
                            let (left_idx, right_idx) = sweep_line_status[sl_idx];
                            let left_ep = get_edgepoints(left_idx);
                            let right_ep = get_edgepoints(right_idx);
                            if !on_left(&left_ep, curr) {
                                assert!(!on_left(&right_ep, curr));
                            } else if on_left(&left_ep, curr) {
                                assert!(on_left(&right_ep, curr));
                            }
                        }
                        for sl_idx in 0..sweep_line_status.len() {
                            let (left_idx, _right_idx) = sweep_line_status[sl_idx];
                            let left_ep = get_edgepoints(left_idx);
                            if on_left(&left_ep, curr) {
                                r = sl_idx;
                                break;
                            }
                        }
                        r
                    };
                    sweep_line_status.insert(entry_index, (curr_idx, prev_idx));
                    update_trapezoids(curr_idx, prev_idx, curr_idx, false);
                    //All start vertices are this way
                }
                PointType::Split => {
                    let entry_index = if sweep_line_status.is_empty() {
                        panic!("This is not possible");
                    } else {
                        let mut r = 0;
                        for sl_idx in 0..sweep_line_status.len() {
                            let (left_idx, right_idx) = sweep_line_status[sl_idx];
                            let left_ep = get_edgepoints(left_idx);
                            let right_ep = get_edgepoints(right_idx);
                            if !on_left(&left_ep, curr) {
                                if on_left(&right_ep, curr) {
                                    r = sl_idx;
                                    break;
                                }
                            }
                        }
                        #[cfg(debug_assertions)]
                        for sl_idx in 0..sweep_line_status.len() {
                            let (left_idx, right_idx) = sweep_line_status[sl_idx];
                            let left_ep = get_edgepoints(left_idx);
                            let right_ep = get_edgepoints(right_idx);
                            if sl_idx < r {
                                assert!(!on_left(&left_ep, curr) && !on_left(&right_ep, curr));
                            } else if sl_idx > r {
                                assert!(on_left(&left_ep, curr) && on_left(&right_ep, curr));
                            }
                        }
                        r
                    };
                    let (ll, rr) = sweep_line_status[entry_index];
                    let (lr, rl) = (prev_idx, curr_idx);
                    sweep_line_status.remove(entry_index);
                    sweep_line_status.insert(entry_index, (rl, rr));
                    sweep_line_status.insert(entry_index, (ll, lr));
                    update_trapezoids(ll, rr, curr_idx, true);
                    update_trapezoids(rl, rr, curr_idx, false);
                    update_trapezoids(ll, lr, curr_idx, false);
                }
                PointType::Merge => {
                    let entry_index = {
                        let mut sl_idx = 0;
                        loop {
                            let (_, right_idx) = sweep_line_status[sl_idx];
                            if right_idx == curr_idx {
                                break sl_idx;
                            }
                            sl_idx += 1; //Note: There will be automatic panic on overflow
                        }
                    };
                    let (ll, lr) = sweep_line_status[entry_index];
                    let (rl, rr) = sweep_line_status[entry_index + 1];
                    debug_assert!(lr == curr_idx);
                    debug_assert!(rl == prev_idx);
                    sweep_line_status.remove(entry_index);
                    sweep_line_status.remove(entry_index);
                    sweep_line_status.insert(entry_index, (ll, rr));
                    update_trapezoids(rl, rr, curr_idx, true);
                    update_trapezoids(ll, lr, curr_idx, true);
                    update_trapezoids(ll, rr, curr_idx, false);
                }
                PointType::End => {
                    let entry_index = {
                        let mut sl_idx = 0;
                        loop {
                            let (_, right_idx) = sweep_line_status[sl_idx];
                            if right_idx == curr_idx {
                                break sl_idx;
                            }
                            sl_idx += 1; //Note: There will be automatic panic on overflow
                        }
                    };
                    let (l, r) = sweep_line_status[entry_index];
                    debug_assert!(l == prev_idx);
                    debug_assert!(r == curr_idx);
                    sweep_line_status.remove(entry_index);
                    update_trapezoids(l, r, curr_idx, true);
                }
                PointType::Regular => {
                    let mut sl_idx = 0;
                    loop {
                        let (left_idx, right_idx) = sweep_line_status[sl_idx];
                        debug_assert_ne!(curr_idx, left_idx);
                        debug_assert_ne!(prev_idx, right_idx);
                        if curr_idx == right_idx {
                            sweep_line_status[sl_idx] = (left_idx, prev_idx);
                            update_trapezoids(left_idx, curr_idx, curr_idx, true);
                            update_trapezoids(left_idx, prev_idx, curr_idx, false);
                            break;
                        }
                        if prev_idx == left_idx {
                            sweep_line_status[sl_idx] = (curr_idx, right_idx);
                            update_trapezoids(prev_idx, right_idx, curr_idx, true);
                            update_trapezoids(curr_idx, right_idx, curr_idx, false);
                            break;
                        }
                        sl_idx += 1; //Note: There will be automatic panic on overflow
                    }
                }
            }
        }
        println!("{:?}", trapezoids_temp);
        #[cfg(debug_assertions)]
        for ((_, _), t) in &trapezoids_temp {
            assert_ne!(t.higher_idx, None);
            assert_ne!(t.lower_idx, None);
        }

        let mut ret = Trapezoidalization(Vec::new());
        for ((l, r), t) in trapezoids_temp {
            let t = TrapezoidalizationRecord {
                left_edge: (l, get_edgepoints(l)),
                right_edge: (r, get_edgepoints(r)),
                top_vertex: (t.higher_idx.unwrap(), &point_list[t.higher_idx.unwrap()]),
                bottom_vertex: (t.lower_idx.unwrap(), &point_list[t.lower_idx.unwrap()]),
            };
            ret.0.push(t);
        }
        ret
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
        for _i in 1..retry_cnt {
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
