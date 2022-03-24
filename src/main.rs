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
        let p = SimplePolygon::gen_rand_hard(6, 1000, 100).unwrap();
        //p.partition_monotone();

        let point_list = p.get_point_list();
        for idx in 0..point_list.len() {
            let next_idx = (idx + 1) % point_list.len();

            let msg = draw::DrawMessage::Edge(
                (point_list[idx].clone(), point_list[next_idx].clone()),
                Colors::RED,
            );
            tx.send(msg).unwrap();
        }
    };
    loop {
        engine();
        thread::sleep(std::time::Duration::from_millis(1000));
        tx.send(draw::DrawMessage::Clear(Colors::BLACK)).unwrap();
    }
}
