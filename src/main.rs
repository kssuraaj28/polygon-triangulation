use piston_window::*;
use polygon::SimplePolygon;
use types::Color;

mod polygon;

fn main() {
    let p = SimplePolygon::gen_rand_hard(10, 1000, 100).unwrap();

    let mut window: PistonWindow = WindowSettings::new("Polygon", [1000, 1000])
        .exit_on_esc(true)
        .build()
        .unwrap();
    let point_list = p.get_point_list();
    while let Some(e) = window.next() {
        let red: Color = [1., 0., 0., 1.];
        window.draw_2d(&e, |c, g, _| {
            clear([1.0; 4], g);
            let points = point_list;
            let mut prev = &points[0];
            for pt in points[1..].iter() {
                line(
                    red,
                    2.,
                    [prev.x as f64, prev.y as f64, pt.x as f64, pt.y as f64],
                    c.transform,
                    g,
                );
                prev = pt;
            }
            line(
                red,
                2.,
                [
                    point_list[0].x as f64,
                    point_list[0].y as f64,
                    point_list[point_list.len() - 1].x as f64,
                    point_list[point_list.len() - 1].y as f64,
                ],
                c.transform,
                g,
            );
        });
    }
}
