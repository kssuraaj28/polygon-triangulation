use crate::polygon::SimplePolygon;
use crate::primitives::Point;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    struct PointKey;
    struct EdgeKey;
    struct FaceKey;
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCEL {
    points: SlotMap<PointKey, DCELPoint>,
    edges: SlotMap<EdgeKey, DCELEdge>,
    faces: SlotMap<FaceKey, DCELFace>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCELPoint {
    parent_key: Option<PointKey>,
    point2d: Point,
    incident_edge: Option<EdgeKey>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCELEdge {
    parent_key: Option<EdgeKey>,
    origin: Option<PointKey>,
    next: Option<EdgeKey>,
    prev: Option<EdgeKey>,
    twin: Option<EdgeKey>,
    incident_face: Option<FaceKey>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DCELFace {
    parent_key: Option<FaceKey>,
    inner: Option<EdgeKey>,
    outer: Option<EdgeKey>,
}

impl DCEL {
    fn split_face(start_edge: DCELEdge, end_vertex: DCELPoint) {}
    pub fn from_simple_polygon(p: &SimplePolygon) -> Self {
        let inp_point_list = p.get_point_list();
        let inp_size = inp_point_list.len();
        let mut ret = DCEL {
            points: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            faces: SlotMap::with_key(),
        };

        let mut point_key_vec = Vec::new();
        let mut edge_key_vec = Vec::new();

        //Creating face placeholders
        let f_inside = ret.faces.insert(DCELFace {
            parent_key: None,
            inner: None,
            outer: None,
        });
        ret.faces[f_inside].parent_key = Some(f_inside);
        let f_outside = ret.faces.insert(DCELFace {
            parent_key: None,
            inner: None,
            outer: None,
        });
        ret.faces[f_outside].parent_key = Some(f_outside);

        //Creating points and edges
        for idx in 0..inp_size {
            let cur_pt = &inp_point_list[idx];
            let p = DCELPoint {
                parent_key: None,
                point2d: cur_pt.clone(),
                incident_edge: None,
            };
            let e = DCELEdge {
                parent_key: None,
                origin: None,
                next: None,
                prev: None,
                twin: None,
                incident_face: None,
            };

            point_key_vec.push(ret.points.insert(p));
            edge_key_vec.push(ret.edges.insert(e));
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

            p.parent_key = Some(curr_point_key);
            p.incident_edge = Some(curr_edge_key);

            e.parent_key = Some(curr_edge_key);
            e.origin = Some(curr_point_key);
            e.next = Some(next_edge_key);
            e.prev = Some(prev_edge_key);
            e.incident_face = Some(f_inside);
        }

        let mut twin_edges = Vec::new();

        for twin_idx in 0..inp_size {
            let e_key = ret.edges.insert(DCELEdge {
                parent_key: None,
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

            e.parent_key = Some(e_key);
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
    fn test_construction() {
        let p = SimplePolygon::gen_rand_hard(5, 1000, 100).unwrap();
        let x = DCEL::from_simple_polygon(&p);
        for (_, x) in x.faces {
            assert_ne!(x.parent_key, None);
            println!("{:?}", x)
        }
        for (_, x) in x.edges {
            assert_ne!(x.parent_key, None);
            assert_ne!(x.origin, None);
            assert_ne!(x.next, None);
            assert_ne!(x.prev, None);
            assert_ne!(x.twin, None);
            println!("{:?}", x)
        }
        for (_, x) in x.points {
            assert_ne!(x.parent_key, None);
            assert_ne!(x.incident_edge, None);
            println!("{:?}", x)
        }
    }
}
