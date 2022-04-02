use dcel::DCEL;
use draw::{Color, Colors};
use polygon::SimplePolygon;
use primitives::DirEdge;
use std::sync::mpsc;
use std::thread;

mod dcel;
mod draw;
mod polygon;
mod primitives;

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || worker(tx));
    draw::drawer(rx);
}

fn clear(tx: &mpsc::Sender<draw::DrawMessage>) {
    tx.send(draw::DrawMessage::Clear(Colors::BLACK, Colors::CYAN, 10))
        .unwrap();
}
fn draw_polygon(tx: &mpsc::Sender<draw::DrawMessage>, polygon: &SimplePolygon, c: Color) {
    let point_list = polygon.get_point_list();
    for idx in 0..point_list.len() {
        let next_idx = (idx + 1) % point_list.len();

        let msg =
            draw::DrawMessage::Edge((point_list[idx].clone(), point_list[next_idx].clone()), c);
        tx.send(msg).unwrap();
    }
    for p in point_list {
        let msg = draw::DrawMessage::Point(p.clone(), Colors::GREEN);
        tx.send(msg).unwrap();
    }
}

fn draw_edge(tx: &mpsc::Sender<draw::DrawMessage>, e: &DirEdge) {
    let msg = draw::DrawMessage::Edge((e.start.clone(), e.end.clone()), Colors::BLUE);
    tx.send(msg).unwrap();
}

fn worker(tx: mpsc::Sender<draw::DrawMessage>) {
    let engine = || {
        let p = SimplePolygon::gen_rand_hard(15, 1000, 100).unwrap();
        println!("Polygon generated");

        clear(&tx);
        draw_polygon(&tx, &p, Colors::RED);
        thread::sleep(std::time::Duration::from_millis(1000));
        let monpart = p.partition_monotone();

        let mut dcel = DCEL::from_simple_polygon(&p);
        dcel.add_internal_diagonals(&monpart);
        println!("Added diagonals");
        for face in dcel.get_internal_faces() {
            println!("Drawing {:?}", face);
            let p = SimplePolygon::from_point_list(dcel.get_point_list(face));

            let edges = p.triangulate_monotone();
            println!("{:?}", p);
            println!("{:?}", edges);
            draw_polygon(&tx, &p, Colors::WHITE);
            for i in edges {
                draw_edge(&tx, &i);
            }
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    };
    loop {
        engine();
        thread::sleep(std::time::Duration::from_millis(1000));
        tx.send(draw::DrawMessage::Clear(Colors::BLACK, Colors::CYAN, 10))
            .unwrap();
    }
}
