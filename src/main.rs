// GroupID-6 (18117106_18114083) - Suraaj K S & Yashaswi Jaiswal
// Date: April 14, 2022
// main.rs - The main runner code
use dcel::DCEL;
use draw::{Color, Colors};
use polygon::SimplePolygon;
use primitives::DirEdge;
use std::collections::HashMap;
use std::env;
use std::sync::mpsc;
use std::thread;

mod dcel;
mod draw;
mod polygon;
mod primitives;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || worker(tx, args[1].parse().unwrap()));
    draw::drawer(rx);
}

fn clear(tx: &mpsc::Sender<draw::DrawMessage>) {
    tx.send(draw::DrawMessage::Clear(Colors::BLACK, Colors::CYAN, 10))
        .unwrap();
}
fn draw_polygon(
    tx: &mpsc::Sender<draw::DrawMessage>,
    polygon: &SimplePolygon,
    lc: Option<Color>,
    pc: Option<Color>,
) {
    let point_list = polygon.get_point_list();
    if lc.is_some() {
        for idx in 0..point_list.len() {
            let next_idx = (idx + 1) % point_list.len();

            let msg = draw::DrawMessage::Edge(
                (point_list[idx].clone(), point_list[next_idx].clone()),
                lc.unwrap(),
            );
            tx.send(msg).unwrap();
        }
    }
    if pc.is_none() {
        return;
    };
    let pc = pc.unwrap();
    for p in point_list {
        let msg = draw::DrawMessage::Point(p.clone(), pc);
        tx.send(msg).unwrap();
    }
}

fn draw_edge(tx: &mpsc::Sender<draw::DrawMessage>, e: &DirEdge, c: Color) {
    let msg = draw::DrawMessage::Edge((e.start.clone(), e.end.clone()), c);
    tx.send(msg).unwrap();
}

fn worker(tx: mpsc::Sender<draw::DrawMessage>, arg: usize) {
    let engine = |sleep_time| {
        let original_p = SimplePolygon::gen_rand_hard(arg, 1000, 100).unwrap();

        clear(&tx);
        draw_polygon(&tx, &original_p, Some(Colors::RED), Some(Colors::GREEN));
        thread::sleep(std::time::Duration::from_millis(sleep_time));
        let monpart = original_p.partition_monotone();
        for e in &monpart {
            draw_edge(&tx, e, Colors::PINK);
        }
        draw_polygon(&tx, &original_p, None, Some(Colors::GREEN));
        thread::sleep(std::time::Duration::from_millis(sleep_time));

        let mut dcel = DCEL::from_simple_polygon(&original_p);
        dcel.add_internal_diagonals(&monpart);

        let mut trg_diagonals = Vec::new();
        for face in dcel.get_internal_faces() {
            let p = SimplePolygon::from_point_list(dcel.get_point_list(face));

            let mut edges = p.triangulate_monotone();

            for e in &edges {
                draw_edge(&tx, e, Colors::YELLOW);
            }
            trg_diagonals.append(&mut edges);
        }
        thread::sleep(std::time::Duration::from_millis(sleep_time));

        clear(&tx);
        dcel.add_internal_diagonals(&trg_diagonals);
        for f in dcel.get_internal_faces() {
            draw_polygon(
                &tx,
                &SimplePolygon::from_point_list(dcel.get_point_list(f)),
                Some(Colors::WHITE),
                Some(Colors::GREEN),
            );
        }

        clear(&tx);
        draw_polygon(&tx, &original_p, None, Some(Colors::GREEN));
        for e in dcel.dual_graph() {
            draw_edge(&tx, &e, Colors::BLUE);
        }
        thread::sleep(std::time::Duration::from_millis(sleep_time));

        clear(&tx);
        for f in dcel.get_internal_faces() {
            draw_polygon(
                &tx,
                &SimplePolygon::from_point_list(dcel.get_point_list(f)),
                Some(Colors::YELLOW),
                Some(Colors::GREEN),
            );
        }

        let color_map = dcel.three_color();
        let mut color_freqs: HashMap<usize, usize> = HashMap::new();
        for (p, i) in &color_map {
            let c = match i {
                1 => {
                    *color_freqs.entry(1).or_insert(0) += 1;
                    Colors::CYAN
                }
                2 => {
                    *color_freqs.entry(2).or_insert(0) += 1;
                    Colors::ORANGE
                }
                3 => {
                    *color_freqs.entry(3).or_insert(0) += 1;
                    Colors::PURPLE
                }
                _ => {
                    panic!("Unknown color. Possible program bug");
                }
            };

            let msg = draw::DrawMessage::Point(p.clone(), c);
            tx.send(msg).unwrap();
        }
        let (&min_clr, _) = color_freqs.iter().min_by_key(|(_, v)| *v).unwrap();
        thread::sleep(std::time::Duration::from_millis(sleep_time));
        clear(&tx);
        draw_polygon(&tx, &original_p, Some(Colors::RED), None);
        for (p, &i) in &color_map {
            if i == min_clr {
                let msg = draw::DrawMessage::Point(p.clone(), Colors::INDIGO);
                tx.send(msg).unwrap();
            }
        }
    };
    loop {
        engine(3000);
        thread::sleep(std::time::Duration::from_millis(2000));
        tx.send(draw::DrawMessage::Clear(Colors::BLACK, Colors::CYAN, 10))
            .unwrap();
    }
}
