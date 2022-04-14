// GroupID-6 (18117106_18114083) - Suraaj K S & Yashaswi Jaiswal
// Date: April 14, 2022
// primitives.rs - Basic code for points and edges
#[derive(Hash, Debug, PartialEq, Eq)]
pub struct Point {
    //TODO: Have generics
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
    pub fn is_higher_than(&self, other: &Self) -> bool {
        if self == other {
            panic!("Cannot use same point");
        }
        if self.y > other.y || (self.y == other.y && self.x > other.x) {
            return true;
        }
        false
    }

    pub fn orientation(p: &Point, q: &Point, r: &Point) -> PointOrientation {
        let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
        if val == 0 {
            return PointOrientation::Collinear;
        } else if val > 0 {
            return PointOrientation::Clockwise;
        } else {
            return PointOrientation::Counterclockwise;
        }
    }

    pub fn clone(&self) -> Point {
        Self {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PointOrientation {
    Clockwise,
    Counterclockwise,
    Collinear,
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub struct DirEdge {
    pub start: Point,
    pub end: Point,
}

impl DirEdge {
    pub fn from_points(start: &Point, end: &Point) -> Self {
        Self {
            start: start.clone(),
            end: end.clone(),
        }
    }
    pub fn intersects(&self, other: &Self) -> bool {
        fn on_segment(e: &DirEdge, q: &Point) -> bool {
            let p = &e.start;
            let r = &e.end;

            use std::cmp::max;
            use std::cmp::min;

            if q.x <= max(p.x, r.x)
                && q.x >= min(p.x, r.x)
                && q.y <= max(p.y, r.y)
                && q.y >= min(p.y, r.y)
            {
                return true;
            } else {
                return false;
            }
        }

        let e1 = self;
        let e2 = other;

        let p1 = &e1.start;
        let q1 = &e1.end;

        let p2 = &e2.start;
        let q2 = &e2.end;

        let o1 = Point::orientation(p1, q1, p2);
        let o2 = Point::orientation(p1, q1, q2);
        let o3 = Point::orientation(p2, q2, p1);
        let o4 = Point::orientation(p2, q2, q1);

        // General case
        if o1 != o2 && o3 != o4 {
            return true;
        }

        if o1 == PointOrientation::Collinear && on_segment(e1, p2) {
            return true;
        };
        if o2 == PointOrientation::Collinear && on_segment(e1, q2) {
            return true;
        };
        if o3 == PointOrientation::Collinear && on_segment(e2, p1) {
            return true;
        };
        if o4 == PointOrientation::Collinear && on_segment(e2, q1) {
            return true;
        };
        false
    }
}
#[cfg(test)]
mod edge_tests {
    use super::*;
    #[test]
    fn test_intersection() {
        fn tester(points: &[isize; 8], expect: bool) {
            let p1 = Point {
                x: points[0],
                y: points[1],
            };
            let q1 = Point {
                x: points[2],
                y: points[3],
            };
            let p2 = Point {
                x: points[4],
                y: points[5],
            };
            let q2 = Point {
                x: points[6],
                y: points[7],
            };
            let e1 = DirEdge::from_points(&p1, &q1);
            let e2 = DirEdge::from_points(&p2, &q2);

            println!("Running e1,e2");
            assert_eq!(e1.intersects(&e2), expect);
            println!("Running e2,e1");
            assert_eq!(e2.intersects(&e1), expect);
        }

        let t = [0, 0, 1, 2, 1, 0, 2, 2];
        tester(&t, false);
        let t = [10, 0, 0, 10, 0, 0, 10, 10];
        tester(&t, true);
        let t = [-5, -5, 0, 0, 1, 1, 10, 10];
        tester(&t, false);
        let t = [0, 0, 100, 0, 0, 0, 1, 1];
        tester(&t, true);
        let t = [0, 0, 100, 0, 50, 0, 1, 1];
        tester(&t, true);
    }
}
