use crate::polygon::SimplePolygon;
use crate::primitives::{DirEdge, Point};
use slotmap::{new_key_type, SlotMap};
use std::collections::{HashMap, HashSet};

new_key_type! {
    struct DCELPointKey;
    struct DCELEdgeKey;
    struct DCELFaceKey;
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCEL {
    points: SlotMap<DCELPointKey, DCELPoint>,
    edges: SlotMap<DCELEdgeKey, DCELEdge>,
    faces: SlotMap<DCELFaceKey, DCELFace>,
    point_hash: HashMap<Point, DCELPointKey>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCELPoint {
    parent_key: DCELPointKey,
    point2d: Point,
    incident_edge: Option<DCELEdgeKey>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCELEdge {
    parent_key: DCELEdgeKey,
    origin: Option<DCELPointKey>,
    next: Option<DCELEdgeKey>,
    prev: Option<DCELEdgeKey>,
    twin: Option<DCELEdgeKey>,
    incident_face: Option<DCELFaceKey>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCELFace {
    parent_key: DCELFaceKey, //TODO - Remove all Options from here
    inner: Option<DCELEdgeKey>,
    outer: Option<DCELEdgeKey>,
}

impl DCEL {
    fn get_dcelpoint_key(&self, p: &Point) -> Option<DCELPointKey> {
        if let Some(x) = self.point_hash.get(p) {
            return Some(*x);
        }
        None
    }

    fn get_next_edge(&self, e: DCELEdgeKey) -> DCELEdgeKey {
        let r = self.edges[e].next.unwrap();
        r
    }

    fn get_twin_edge(&self, e: DCELEdgeKey) -> DCELEdgeKey {
        self.edges[e].twin.unwrap()
    }

    fn get_prev_edge(&self, e: DCELEdgeKey) -> DCELEdgeKey {
        let r = self.edges[e].prev.unwrap();
        r
    }

    fn get_origin_point(&self, e: DCELEdgeKey) -> DCELPointKey {
        self.edges[e].origin.unwrap()
    }

    fn get_common_face(&self, p1: DCELPointKey, p2: DCELPointKey) -> Option<DCELFaceKey> {
        if p1 == p2 {
            panic!("Why same points?");
        }
        for (facekey, face) in &self.faces {
            let start_edge = face.outer;
            if start_edge == None {
                continue;
            }
            let start_edge = start_edge.unwrap();

            let mut points_found = 0;

            let check_pts = |e| {
                let e_pt = self.get_origin_point(e);

                if e_pt == p1 || e_pt == p2 {
                    true
                } else {
                    false
                }
            };
            if check_pts(start_edge) {
                points_found += 1
            }
            let mut curr_edge = self.get_next_edge(start_edge);
            loop {
                if curr_edge == start_edge {
                    break;
                }
                if check_pts(curr_edge) {
                    points_found += 1
                }
                if points_found == 2 {
                    return Some(facekey);
                }

                curr_edge = self.get_next_edge(curr_edge);
            }
        }
        return None;
    }
    fn check_consistency(&self) {
        for (e, _) in &self.edges {
            assert_eq!(e, self.get_next_edge(self.get_prev_edge(e)));
            assert_eq!(e, self.get_prev_edge(self.get_next_edge(e)));
        }
        for (f, _) in &self.faces {
            let e = self.faces[f].outer;
            if e.is_none() {
                continue;
            }
            let e = e.unwrap();
            let mut curr_edge = e;
            loop {
                assert_eq!(f, self.edges[e].incident_face.unwrap());
                curr_edge = self.get_next_edge(curr_edge);
                if curr_edge == e {
                    break;
                };
            }
        }
    }
    fn print_edges(&self) {
        for (e, _) in &self.edges {
            println!(
                "{:?} {:?} {:?} {:?} {:?}",
                e,
                self.get_next_edge(e),
                self.get_prev_edge(e),
                self.get_twin_edge(e),
                self.edges[e].incident_face.unwrap(),
            );
        }
    }

    fn split_with_triangle(&mut self, e_int: DCELEdgeKey) {
        #[cfg(debug_assertions)]
        {
            println!("In triangle split, {:?}", e_int);
            self.print_edges();
            self.check_consistency();
        }
        let e_prev = e_int;
        let e_next = self.get_prev_edge(e_int);
        let e_twin_next = self.get_next_edge(e_int);
        let e_twin_prev = self.get_prev_edge(e_next);

        let e_orig = self.get_origin_point(e_twin_next);
        let e_twin_orig = self.get_origin_point(e_next);

        let e_new = self.edges.insert_with_key(|k| DCELEdge {
            parent_key: k,
            origin: Some(e_orig),
            next: Some(e_next),
            prev: Some(e_prev),
            twin: None,
            incident_face: None,
        });
        let e_new_twin = self.edges.insert_with_key(|k| DCELEdge {
            parent_key: k,
            origin: Some(e_twin_orig),
            next: Some(e_twin_next),
            prev: Some(e_twin_prev),
            twin: None,
            incident_face: None,
        });

        self.edges[e_new].twin = Some(e_new_twin);
        self.edges[e_new_twin].twin = Some(e_new);

        self.edges[e_prev].next = Some(e_new);
        self.edges[e_next].prev = Some(e_new);
        self.edges[e_twin_prev].next = Some(e_new_twin);
        self.edges[e_twin_next].prev = Some(e_new_twin);

        let f = self.faces.insert_with_key(|k| DCELFace {
            parent_key: k,
            inner: None,
            outer: Some(e_new),
        });

        self.edges[e_new].incident_face = Some(f);
        self.edges[e_next].incident_face = Some(f);
        self.edges[e_prev].incident_face = Some(f);
        let other_face = self.edges[e_twin_next].incident_face.unwrap();
        self.faces[other_face].outer = Some(e_new_twin);

        self.edges[e_new_twin].incident_face = Some(other_face);
        #[cfg(debug_assertions)]
        {
            println!("After triangle split");
            self.print_edges();
            self.check_consistency();
        }
    }
    //This is terribly inefficient, but I don't care. Because I'm a thug
    fn split_face(&mut self, p1: DCELPointKey, p2: DCELPointKey) -> bool {
        #[cfg(debug_assertions)]
        self.print_edges();

        let common_face = self.get_common_face(p1, p2);
        if common_face.is_none() {
            return false;
        }
        let start_edge = self.faces[common_face.unwrap()].outer.unwrap();
        let (e1_next, e2_next) = {
            let mut curr_edge = start_edge;
            let (mut e1, mut e2) = (None, None);
            loop {
                if self.get_origin_point(curr_edge) == p1 {
                    e1 = Some(curr_edge);
                }
                if self.get_origin_point(curr_edge) == p2 {
                    e2 = Some(curr_edge);
                }
                if e1.is_some() && e2.is_some() {
                    break (e1.unwrap(), e2.unwrap());
                }
                curr_edge = self.get_next_edge(curr_edge);
                if curr_edge == start_edge {
                    dbg!(&self, p1, p2);
                    panic!("This is not supposed to happen");
                }
            }
        };

        if self.get_next_edge(e1_next) == e2_next || self.get_next_edge(e2_next) == e1_next {
            //This is a triangle
            return true;
        }

        if self.get_next_edge(self.get_next_edge(e1_next)) == e2_next {
            self.split_with_triangle(self.get_next_edge(e1_next));
            return true;
        }
        if self.get_next_edge(self.get_next_edge(e2_next)) == e1_next {
            self.split_with_triangle(self.get_next_edge(e2_next));
            return true;
        }

        let e2_prev = self.get_prev_edge(e1_next);
        let e1_prev = self.get_prev_edge(e2_next);

        //println!("{:?} {:?}", e1_next, e2_next);
        //println!("{:?} {:?}", e1_prev, e2_prev);
        //println!("-----");

        let e1 = self.edges.insert_with_key(|k| DCELEdge {
            parent_key: k,
            origin: Some(p2),
            next: Some(e1_next),
            prev: Some(e1_prev),
            twin: None,
            incident_face: None,
        });
        let e2 = self.edges.insert_with_key(|k| DCELEdge {
            parent_key: k,
            origin: Some(p1),
            next: Some(e2_next),
            prev: Some(e2_prev),
            twin: None,
            incident_face: None,
        });

        self.edges[e1].twin = Some(e2);
        self.edges[e2].twin = Some(e1);

        let f1 = self.faces.insert_with_key(|k| DCELFace {
            parent_key: k,
            inner: None,
            outer: Some(e1),
        });
        let f2 = self.faces.insert_with_key(|k| DCELFace {
            parent_key: k,
            inner: None,
            outer: Some(e2),
        });

        self.edges[e1_next].prev = Some(e1);
        self.edges[e2_next].prev = Some(e2);
        self.edges[e1_prev].next = Some(e1);
        self.edges[e2_prev].next = Some(e2);

        let mut update_faces = |e, f| {
            let mut curr_edge = e;
            loop {
                self.edges[curr_edge].incident_face = Some(f);
                curr_edge = self.get_next_edge(curr_edge);
                if curr_edge == e {
                    break;
                }
            }
        };
        update_faces(e1, f1);
        update_faces(e2, f2);

        self.faces.remove(common_face.unwrap());
        true
    }

    pub fn get_external_face(&self) -> &DCELFace {
        for (_, f) in &self.faces {
            if f.outer.is_none() {
                assert!(f.inner.is_some());
                return f;
            }
        }
        panic!("External face not found!");
    }

    pub fn get_internal_faces(&self) -> Vec<&DCELFace> {
        let mut r = Vec::new();
        for (_, f) in &self.faces {
            if f.inner.is_none() {
                assert!(f.outer.is_some());
                r.push(f)
            } else {
                assert!(f.outer.is_none())
            }
        }
        r
    }

    pub fn three_color(&self) -> HashMap<Point, usize> {
        let faces = &self.get_internal_faces();
        let external_face = self.get_external_face().parent_key;
        let mut adjacent_faces = HashMap::new();
        for f in faces {
            let start_edge = f.outer.unwrap();
            assert_eq!(
                start_edge,
                self.get_next_edge(self.get_next_edge(self.get_next_edge(start_edge)))
            );
            let e1 = self.get_twin_edge(start_edge);
            let e2 = self.get_twin_edge(self.get_next_edge(start_edge));
            let e3 = self.get_twin_edge(self.get_prev_edge(start_edge));

            let f1 = self.edges[e1].incident_face.unwrap();
            let f2 = self.edges[e2].incident_face.unwrap();
            let f3 = self.edges[e3].incident_face.unwrap();
            let transform = |x| {
                if x == external_face {
                    None
                } else {
                    Some(x)
                }
            };
            let (f1, f2, f3) = (transform(f1), transform(f2), transform(f3));
            adjacent_faces.insert(f.parent_key, [f1, f2, f3]);
        }

        fn recursive_engine(
            curr_face: DCELFaceKey,
            parent_face: DCELFaceKey,
            adjacent_faces: &HashMap<DCELFaceKey, [Option<DCELFaceKey>; 3]>,
            //adjacent_faces: &HashMap,
            coloring: &mut HashMap<DCELPointKey, usize>,
            dcel: &DCEL,
        ) {
            #[cfg(debug_assertions)]
            {
                println!("---------------------");
                println!("{:?} {:?} {:?}", curr_face, parent_face, coloring);
            }
            let pts = DCEL::get_pointkey_list(dcel, curr_face);
            let mut forbidden_colors = HashSet::new();
            let mut new_pts = HashSet::new();
            for p in pts {
                if coloring.contains_key(&p) {
                    forbidden_colors.insert(coloring[&p]);
                } else {
                    new_pts.insert(p);
                }
            }
            #[cfg(debug_assertions)]
            {
                println!("{:?} {:?}", new_pts, forbidden_colors);
            }
            for p in new_pts {
                if !forbidden_colors.contains(&1) {
                    coloring.insert(p, 1);
                    forbidden_colors.insert(1);
                    continue;
                };
                if !forbidden_colors.contains(&2) {
                    coloring.insert(p, 2);
                    forbidden_colors.insert(2);
                    continue;
                };
                if !forbidden_colors.contains(&3) {
                    coloring.insert(p, 3);
                    forbidden_colors.insert(3);
                    continue;
                };
                panic!("Cannot three color!");
            }

            for i in adjacent_faces[&curr_face] {
                if i.is_none() {
                    continue;
                }
                let f = i.unwrap();
                if parent_face == f {
                    continue;
                }
                recursive_engine(f, curr_face, adjacent_faces, coloring, dcel);
            }
        }
        let mut tempret = HashMap::new();
        recursive_engine(
            faces[0].parent_key,
            faces[0].parent_key,
            &adjacent_faces,
            &mut tempret,
            self,
        );
        debug_assert_eq!(self.points.len(), tempret.len());

        let mut ret = HashMap::new();
        for (k, v) in tempret {
            let p2d = self.points[k].point2d.clone();
            ret.insert(p2d, v);
        }
        ret
    }

    fn get_pointkey_list(&self, f: DCELFaceKey) -> Vec<DCELPointKey> {
        let mut r = Vec::new();
        let face = &self.faces[f];
        let start_edge = face.outer.unwrap();
        {
            let mut curr_edge = start_edge;
            loop {
                let p = self.get_origin_point(curr_edge);
                r.push(p);
                curr_edge = self.get_next_edge(curr_edge);
                if curr_edge == start_edge {
                    break;
                }
            }
        }
        r
    }
    pub fn get_point_list(&self, face: &DCELFace) -> Vec<Point> {
        let x = self.get_pointkey_list(face.parent_key);
        x.into_iter()
            .map(|x| self.points[x].point2d.clone())
            .collect()
    }

    pub fn add_internal_diagonals(&mut self, diagonals: &Vec<DirEdge>) {
        let mut int_diagonals = Vec::new();
        for e in diagonals {
            let p1 = self.get_dcelpoint_key(&e.start).unwrap();
            let p2 = self.get_dcelpoint_key(&e.end).unwrap();
            int_diagonals.push((p1, p2));
        }
        let int_diagonals = int_diagonals;
        for (p1, p2) in int_diagonals {
            self.split_face(p1, p2);
        }
    }

    pub fn from_simple_polygon(p: &SimplePolygon) -> Self {
        let inp_point_list = p.get_point_list();
        let inp_size = inp_point_list.len();
        let mut ret = DCEL {
            points: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            faces: SlotMap::with_key(),
            point_hash: HashMap::new(),
        };

        let mut point_key_vec = Vec::new();
        let mut edge_key_vec = Vec::new();

        //Creating face placeholders
        let f_inside = ret.faces.insert_with_key(|k| DCELFace {
            parent_key: k,
            inner: None,
            outer: None,
        });
        let f_outside = ret.faces.insert_with_key(|k| DCELFace {
            parent_key: k,
            inner: None,
            outer: None,
        });

        //Creating points and edges
        for idx in 0..inp_size {
            let cur_pt = &inp_point_list[idx];
            let p = ret.points.insert_with_key(|k| DCELPoint {
                parent_key: k,
                point2d: cur_pt.clone(),
                incident_edge: None,
            });
            ret.point_hash.insert(cur_pt.clone(), p);
            let e = ret.edges.insert_with_key(|k| DCELEdge {
                parent_key: k,
                origin: None,
                next: None,
                prev: None,
                twin: None,
                incident_face: None,
            });

            point_key_vec.push(p);
            edge_key_vec.push(e);
        }

        for idx in 0..inp_size {
            let next_idx = if idx == inp_size - 1 { 0 } else { idx + 1 };
            let prev_idx = if idx == 0 { inp_size - 1 } else { idx - 1 };

            let curr_point_key = point_key_vec[idx];
            let curr_edge_key = edge_key_vec[idx];
            let prev_edge_key = edge_key_vec[prev_idx];
            let next_edge_key = edge_key_vec[next_idx];

            let p = &mut ret.points[point_key_vec[idx]];
            let e = &mut ret.edges[edge_key_vec[idx]];

            p.incident_edge = Some(curr_edge_key);

            e.origin = Some(curr_point_key);
            e.next = Some(next_edge_key);
            e.prev = Some(prev_edge_key);
            e.incident_face = Some(f_inside);
        }

        let mut twin_edges = Vec::new();

        for twin_idx in 0..inp_size {
            let e_key = ret.edges.insert_with_key(|k| DCELEdge {
                parent_key: k,
                origin: None,
                next: None,
                prev: None,
                twin: None,
                incident_face: None,
            });
            twin_edges.push(e_key);

            let e = &mut ret.edges[e_key];

            let origin_idx = if twin_idx == inp_size - 1 {
                0
            } else {
                twin_idx + 1
            };

            let twin_key = edge_key_vec[twin_idx];
            let origin_point_key = point_key_vec[origin_idx];

            e.origin = Some(origin_point_key);
            e.next = None;
            e.prev = None;
            e.twin = Some(twin_key);
            e.incident_face = Some(f_outside);

            ret.edges[twin_key].twin = Some(e_key);
        }
        assert_eq!(twin_edges.len(), inp_size);

        for edge_key in &twin_edges {
            let e = &ret.edges[edge_key.clone()];
            let e_twin = &ret.edges[e.twin.unwrap()];
            let e_next = ret.edges[e_twin.prev.unwrap()].twin;
            let e_prev = ret.edges[e_twin.next.unwrap()].twin;
            let e = &mut ret.edges[edge_key.clone()];
            e.next = e_next;
            e.prev = e_prev;
        }
        ret.faces[f_inside].outer = Some(edge_key_vec[0]);
        ret.faces[f_outside].inner = Some(twin_edges[0]);
        ret
    }
}

#[cfg(test)]
mod dcel_tests {
    use super::*;
    use crate::SimplePolygon;
    #[test]
    fn test_split() {
        let p = SimplePolygon::gen_rand_hard(5, 1000, 100).unwrap();
        let mut x = DCEL::from_simple_polygon(&p);

        let pts = &x.points;
        let mut iter = pts.into_iter();
        iter.next();
        let p1 = iter.next().unwrap().0;
        iter.next();
        iter.next();
        let p4 = iter.next().unwrap().0;
        dbg!(x.get_common_face(p1, p4));

        x.split_face(p1, p4);
        for (_, x) in x.faces {
            println!("{:?}", x)
        }
        for (_, x) in x.edges {
            println!("{:?}", x)
        }
        for (_, x) in x.points {
            println!("{:?}", x)
        }
    }
    #[test]
    fn test_construction() {
        let p = SimplePolygon::gen_rand_hard(5, 1000, 100).unwrap();
        let x = DCEL::from_simple_polygon(&p);
        for (_, x) in x.faces {
            println!("{:?}", x)
        }
        for (_, x) in x.edges {
            assert_ne!(x.origin, None);
            assert_ne!(x.next, None);
            assert_ne!(x.prev, None);
            assert_ne!(x.twin, None);
            println!("{:?}", x)
        }
        for (_, x) in x.points {
            assert_ne!(x.incident_edge, None);
            println!("{:?}", x)
        }
    }
}
