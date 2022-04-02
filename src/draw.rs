use crate::primitives::Point;
use piston_window::*;
use std::sync::mpsc;

pub type Color = [f32; 4];

#[non_exhaustive]
pub struct Colors;

#[allow(unused)]
impl Colors {
    pub const PINK: Color = [1., 0.4, 0.7, 1.];
    pub const ORANGE: Color = [1., 0.4, 0., 1.];
    pub const YELLOW: Color = [1., 1., 0., 1.];
    pub const INDIGO: Color = [0.5, 0.5, 1.0, 1.];
    pub const PURPLE: Color = [1., 0., 1., 1.];
    pub const RED: Color = [1., 0., 0., 1.];
    pub const GREEN: Color = [0., 1., 0., 1.];
    pub const BLUE: Color = [0., 0., 1., 1.];
    pub const CYAN: Color = [0., 1., 1., 1.];
    pub const BLACK: Color = [0., 0., 0., 1.];
    pub const WHITE: Color = [1., 1., 1., 1.];
}

#[allow(unused)]
pub enum DrawMessage {
    Point(Point, Color),
    Edge((Point, Point), Color),
    Clear(Color, Color, usize),
}

pub fn drawer(rx: mpsc::Receiver<DrawMessage>) {
    const X_MAX: f64 = 1000.;
    const Y_MAX: f64 = 1000.;
    let mut window: PistonWindow = WindowSettings::new("Polygon", [X_MAX, Y_MAX])
        .exit_on_esc(true)
        .build()
        .unwrap();

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _| {
            while let Ok(m) = rx.try_recv() {
                match m {
                    DrawMessage::Point(pt, clr) => {
                        ellipse(
                            clr,
                            ellipse::circle(pt.x as f64, Y_MAX - pt.y as f64, 6.),
                            c.transform,
                            g,
                        );
                    }
                    DrawMessage::Edge((pt, prev), clr) => {
                        line(
                            clr,
                            2.,
                            [
                                prev.x as f64,
                                Y_MAX - (prev.y as f64),
                                pt.x as f64,
                                Y_MAX - (pt.y as f64),
                            ],
                            c.transform,
                            g,
                        );
                    }
                    DrawMessage::Clear(x, y, z) => {
                        let bg_color = x;
                        let grid_color = y;
                        let steps = z;
                        clear(bg_color, g);
                        let x_gran = X_MAX / steps as f64;
                        let y_gran = X_MAX / steps as f64;
                        //TODO: We might be able to use polygon method
                        for i in 0..steps {
                            line(
                                grid_color,
                                0.5,
                                [0., y_gran * (i as f64), X_MAX, y_gran * i as f64],
                                c.transform,
                                g,
                            );
                            line(
                                grid_color,
                                0.5,
                                [
                                    i as f64 * x_gran,
                                    Y_MAX - (0 as f64),
                                    i as f64 * x_gran,
                                    Y_MAX - (Y_MAX as f64),
                                ],
                                c.transform,
                                g,
                            );
                        }
                    }
                }
            }
        });
    }
}
