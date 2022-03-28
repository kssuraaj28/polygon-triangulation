use draw::Colors;
use polygon::SimplePolygon;
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

fn worker(tx: mpsc::Sender<draw::DrawMessage>) {
    let engine = || {
        let p = SimplePolygon::gen_rand_hard(30, 1000, 100).unwrap();
        println!("Polygon generated");
        tx.send(draw::DrawMessage::Clear(Colors::BLACK, Colors::CYAN, 10))
            .unwrap();
        let point_list = p.get_point_list();
        for idx in 0..point_list.len() {
            let next_idx = (idx + 1) % point_list.len();

            let msg = draw::DrawMessage::Edge(
                (point_list[idx].clone(), point_list[next_idx].clone()),
                Colors::RED,
            );
            tx.send(msg).unwrap();
        }
        for e in p.partition_monotone() {
            let msg = draw::DrawMessage::Edge((e.start.clone(), e.end.clone()), Colors::BLUE);
            tx.send(msg).unwrap();
        }
        for p in point_list {
            let msg = draw::DrawMessage::Point(p.clone(), Colors::BLUE);
            tx.send(msg).unwrap();
        }
        println!("{:?}", p.get_point_list());
    };
    //loop {
    engine();
    //  thread::sleep(std::time::Duration::from_millis(1000));
    //  tx.send(draw::DrawMessage::Clear(Colors::BLACK, Colors::CYAN, 10))
    //      .unwrap();
    // }
}
