//! Vector icons drawn with the egui painter.
//!
//! Mercury ships no icon font and no emoji: every glyph here is a handful of
//! points on a 24x24 grid, scaled into whatever rect it is asked to fill. That
//! keeps icons crisp at any size, correctly colored in both palettes, and
//! identical on every platform (emoji are not).

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

/// One primitive on the 24x24 design grid.
enum Geo {
    /// Open polyline.
    Line(&'static [(f32, f32)]),
    /// Closed polyline.
    Shape(&'static [(f32, f32)]),
    /// Filled polygon.
    Solid(&'static [(f32, f32)]),
    /// Stroked circle: center, radius.
    Ring((f32, f32), f32),
    /// Filled circle: center, radius.
    Dot((f32, f32), f32),
    /// Stroked rounded rect: min, max, radius.
    Frame((f32, f32), (f32, f32), f32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    ChevronRight,
    ChevronDown,
    Folder,
    File,
    Search,
    Plus,
    Trash,
    Pencil,
    Copy,
    Download,
    Sparkle,
    Check,
    Close,
    Clock,
    Play,
    Alert,
    Sun,
    Moon,
    Terminal,
    ExternalLink,
    Keyboard,
    Layers,
    Dot,
    Inbox,
    Bolt,
    PanelLeft,
    Ellipsis,
    Image,
    Music,
    Film,
    Package,
    Paperclip,
}

impl Icon {
    /// Every icon, so the geometry check can't silently miss a new one.
    #[cfg(test)]
    pub const ALL: [Icon; 32] = [
        Icon::ChevronRight,
        Icon::ChevronDown,
        Icon::Folder,
        Icon::File,
        Icon::Search,
        Icon::Plus,
        Icon::Trash,
        Icon::Pencil,
        Icon::Copy,
        Icon::Download,
        Icon::Sparkle,
        Icon::Check,
        Icon::Close,
        Icon::Clock,
        Icon::Play,
        Icon::Alert,
        Icon::Sun,
        Icon::Moon,
        Icon::Terminal,
        Icon::ExternalLink,
        Icon::Keyboard,
        Icon::Layers,
        Icon::Dot,
        Icon::Inbox,
        Icon::Bolt,
        Icon::PanelLeft,
        Icon::Ellipsis,
        Icon::Image,
        Icon::Music,
        Icon::Film,
        Icon::Package,
        Icon::Paperclip,
    ];

    fn geometry(self) -> &'static [Geo] {
        use Geo::*;
        match self {
            Icon::ChevronRight => &[Line(&[(9.5, 5.0), (16.5, 12.0), (9.5, 19.0)])],
            Icon::ChevronDown => &[Line(&[(5.0, 9.5), (12.0, 16.5), (19.0, 9.5)])],
            Icon::Folder => &[Shape(&[
                (3.0, 7.0),
                (9.4, 7.0),
                (11.4, 9.6),
                (21.0, 9.6),
                (21.0, 19.5),
                (3.0, 19.5),
            ])],
            Icon::File => &[
                Shape(&[
                    (5.0, 3.0),
                    (14.0, 3.0),
                    (19.0, 8.0),
                    (19.0, 21.0),
                    (5.0, 21.0),
                ]),
                Line(&[(14.0, 3.0), (14.0, 8.0), (19.0, 8.0)]),
            ],
            Icon::Search => &[Ring((10.5, 10.5), 6.5), Line(&[(15.3, 15.3), (20.0, 20.0)])],
            Icon::Plus => &[
                Line(&[(12.0, 5.0), (12.0, 19.0)]),
                Line(&[(5.0, 12.0), (19.0, 12.0)]),
            ],
            Icon::Trash => &[
                Line(&[(3.5, 6.5), (20.5, 6.5)]),
                Line(&[(9.0, 6.5), (9.0, 3.5), (15.0, 3.5), (15.0, 6.5)]),
                Line(&[(5.5, 6.5), (6.6, 20.5), (17.4, 20.5), (18.5, 6.5)]),
                Line(&[(10.0, 10.5), (10.0, 16.5)]),
                Line(&[(14.0, 10.5), (14.0, 16.5)]),
            ],
            Icon::Pencil => &[
                Shape(&[
                    (3.5, 20.5),
                    (4.5, 16.0),
                    (16.0, 4.5),
                    (19.5, 8.0),
                    (8.0, 19.5),
                ]),
                Line(&[(14.5, 6.0), (18.0, 9.5)]),
            ],
            Icon::Copy => &[
                Frame((8.5, 8.5), (20.5, 20.5), 2.5),
                Line(&[
                    (16.0, 8.5),
                    (16.0, 3.5),
                    (3.5, 3.5),
                    (3.5, 16.0),
                    (8.5, 16.0),
                ]),
            ],
            Icon::Download => &[
                Line(&[(12.0, 3.0), (12.0, 15.5)]),
                Line(&[(6.8, 10.3), (12.0, 15.5), (17.2, 10.3)]),
                Line(&[(4.0, 20.5), (20.0, 20.5)]),
            ],
            Icon::Sparkle => &[Shape(&[
                (12.0, 2.5),
                (14.0, 10.0),
                (21.5, 12.0),
                (14.0, 14.0),
                (12.0, 21.5),
                (10.0, 14.0),
                (2.5, 12.0),
                (10.0, 10.0),
            ])],
            Icon::Check => &[Line(&[(4.5, 12.5), (9.5, 17.5), (19.5, 6.5)])],
            Icon::Close => &[
                Line(&[(6.0, 6.0), (18.0, 18.0)]),
                Line(&[(18.0, 6.0), (6.0, 18.0)]),
            ],
            Icon::Clock => &[
                Ring((12.0, 12.0), 8.8),
                Line(&[(12.0, 6.5), (12.0, 12.2), (16.4, 14.6)]),
            ],
            Icon::Play => &[Solid(&[(8.0, 4.8), (19.2, 12.0), (8.0, 19.2)])],
            Icon::Alert => &[
                Shape(&[(12.0, 3.0), (22.0, 20.2), (2.0, 20.2)]),
                Line(&[(12.0, 9.5), (12.0, 14.5)]),
                Dot((12.0, 17.4), 1.15),
            ],
            Icon::Sun => &[
                Ring((12.0, 12.0), 4.8),
                Line(&[(12.0, 1.6), (12.0, 4.4)]),
                Line(&[(12.0, 19.6), (12.0, 22.4)]),
                Line(&[(1.6, 12.0), (4.4, 12.0)]),
                Line(&[(19.6, 12.0), (22.4, 12.0)]),
                Line(&[(4.7, 4.7), (6.7, 6.7)]),
                Line(&[(17.3, 17.3), (19.3, 19.3)]),
                Line(&[(4.7, 19.3), (6.7, 17.3)]),
                Line(&[(17.3, 6.7), (19.3, 4.7)]),
            ],
            Icon::Moon => &[Shape(&[
                (16.4, 4.9),
                (11.5, 3.4),
                (5.4, 5.9),
                (2.9, 12.0),
                (5.4, 18.1),
                (11.5, 20.6),
                (16.4, 19.1),
                (12.5, 18.7),
                (9.5, 14.8),
                (9.0, 12.0),
                (9.5, 9.2),
                (12.5, 5.3),
            ])],
            Icon::Terminal => &[
                Frame((2.5, 3.5), (21.5, 20.5), 3.0),
                Line(&[(6.5, 9.0), (10.5, 12.5), (6.5, 16.0)]),
                Line(&[(13.0, 16.2), (18.0, 16.2)]),
            ],
            Icon::ExternalLink => &[
                Line(&[
                    (19.0, 13.5),
                    (19.0, 19.5),
                    (4.5, 19.5),
                    (4.5, 5.0),
                    (10.5, 5.0),
                ]),
                Line(&[(14.0, 4.0), (20.0, 4.0), (20.0, 10.0)]),
                Line(&[(20.0, 4.0), (11.5, 12.5)]),
            ],
            Icon::Keyboard => &[
                Frame((2.0, 6.0), (22.0, 18.5), 2.5),
                Dot((6.0, 10.5), 0.95),
                Dot((10.0, 10.5), 0.95),
                Dot((14.0, 10.5), 0.95),
                Dot((18.0, 10.5), 0.95),
                Dot((6.0, 14.5), 0.95),
                Dot((18.0, 14.5), 0.95),
                Line(&[(9.6, 14.5), (14.4, 14.5)]),
            ],
            Icon::Layers => &[
                Shape(&[(12.0, 2.6), (21.0, 7.2), (12.0, 11.8), (3.0, 7.2)]),
                Line(&[(3.0, 12.2), (12.0, 16.8), (21.0, 12.2)]),
                Line(&[(3.0, 16.4), (12.0, 21.0), (21.0, 16.4)]),
            ],
            Icon::Dot => &[Dot((12.0, 12.0), 4.4)],
            Icon::Inbox => &[
                Shape(&[
                    (3.0, 12.0),
                    (6.4, 4.5),
                    (17.6, 4.5),
                    (21.0, 12.0),
                    (21.0, 19.5),
                    (3.0, 19.5),
                ]),
                Line(&[
                    (3.0, 12.0),
                    (8.2, 12.0),
                    (9.6, 15.2),
                    (14.4, 15.2),
                    (15.8, 12.0),
                    (21.0, 12.0),
                ]),
            ],
            Icon::Bolt => &[Shape(&[
                (13.6, 2.0),
                (4.0, 13.6),
                (11.0, 13.6),
                (10.4, 22.0),
                (20.0, 10.4),
                (13.0, 10.4),
            ])],
            Icon::PanelLeft => &[
                Frame((2.5, 4.0), (21.5, 20.0), 3.0),
                Line(&[(9.5, 4.0), (9.5, 20.0)]),
            ],
            Icon::Ellipsis => &[
                Dot((5.5, 12.0), 1.6),
                Dot((12.0, 12.0), 1.6),
                Dot((18.5, 12.0), 1.6),
            ],
            Icon::Image => &[
                Frame((3.0, 4.5), (21.0, 19.5), 2.5),
                Ring((8.6, 9.8), 1.9),
                Line(&[
                    (3.4, 17.6),
                    (9.6, 11.6),
                    (14.4, 16.4),
                    (17.0, 13.8),
                    (20.6, 17.4),
                ]),
            ],
            Icon::Music => &[
                Line(&[(9.2, 17.6), (9.2, 4.4), (19.2, 2.4), (19.2, 15.6)]),
                Ring((6.6, 17.6), 2.6),
                Ring((16.6, 15.6), 2.6),
            ],
            Icon::Film => &[
                Frame((2.5, 4.5), (21.5, 19.5), 2.5),
                Line(&[(7.6, 4.5), (7.6, 19.5)]),
                Line(&[(16.4, 4.5), (16.4, 19.5)]),
                Line(&[(2.5, 12.0), (21.5, 12.0)]),
            ],
            Icon::Package => &[
                Shape(&[
                    (12.0, 2.4),
                    (21.0, 7.2),
                    (21.0, 16.8),
                    (12.0, 21.6),
                    (3.0, 16.8),
                    (3.0, 7.2),
                ]),
                Line(&[(3.0, 7.2), (12.0, 12.0), (21.0, 7.2)]),
                Line(&[(12.0, 12.0), (12.0, 21.6)]),
            ],
            Icon::Paperclip => &[Line(&[
                (18.6, 11.4),
                (11.0, 19.0),
                (6.4, 14.4),
                (14.6, 6.2),
                (17.8, 9.4),
                (9.8, 17.4),
            ])],
        }
    }
}

/// Paint `icon` centered in `rect`, scaled to fit its shortest side.
pub fn paint(painter: &egui::Painter, icon: Icon, rect: Rect, color: Color32) {
    let size = rect.width().min(rect.height());
    let scale = size / 24.0;
    let origin = rect.center() - Vec2::splat(size / 2.0);
    let p = |(x, y): (f32, f32)| origin + Vec2::new(x * scale, y * scale);
    let pts = |list: &'static [(f32, f32)]| list.iter().copied().map(p).collect::<Vec<Pos2>>();
    // 2.0 units on the 24-grid, never so thin it disappears on a hidpi screen
    let stroke = Stroke::new((scale * 2.0).max(1.0), color);

    for geo in icon.geometry() {
        match *geo {
            Geo::Line(list) => {
                painter.add(egui::Shape::line(pts(list), stroke));
            }
            Geo::Shape(list) => {
                painter.add(egui::Shape::closed_line(pts(list), stroke));
            }
            Geo::Solid(list) => {
                painter.add(egui::Shape::convex_polygon(pts(list), color, Stroke::NONE));
            }
            Geo::Ring(center, r) => {
                painter.circle_stroke(p(center), r * scale, stroke);
            }
            Geo::Dot(center, r) => {
                painter.circle_filled(p(center), r * scale, color);
            }
            Geo::Frame(min, max, r) => {
                painter.rect_stroke(
                    Rect::from_min_max(p(min), p(max)),
                    r * scale,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
}

/// A spinner drawn as a sweeping arc — the in-flight indicator.
pub fn spinner(painter: &egui::Painter, rect: Rect, color: Color32, time: f64) {
    let radius = rect.width().min(rect.height()) / 2.0 - 1.5;
    let center = rect.center();
    let head = (time * 2.4) as f32 % std::f32::consts::TAU;
    let points: Vec<Pos2> = (0..=24)
        .map(|i| {
            let a = head + i as f32 / 24.0 * 4.2;
            center + Vec2::new(a.cos(), a.sin()) * radius
        })
        .collect();
    painter.add(egui::Shape::line(
        points,
        Stroke::new((radius * 0.32).max(1.2), color),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards against a typo'd geometry table: egui panics on a 1-point
    /// polyline, and an icon with no shapes silently renders nothing.
    #[test]
    fn every_icon_has_drawable_geometry() {
        for icon in Icon::ALL {
            let geometry = icon.geometry();
            assert!(!geometry.is_empty(), "{icon:?} draws nothing");
            for geo in geometry {
                if let Geo::Line(p) | Geo::Shape(p) | Geo::Solid(p) = geo {
                    assert!(p.len() >= 2, "{icon:?} has a degenerate path");
                    for (x, y) in *p {
                        assert!(
                            (0.0..=24.0).contains(x) && (0.0..=24.0).contains(y),
                            "{icon:?} draws outside the 24x24 grid at ({x}, {y})"
                        );
                    }
                }
            }
        }
    }
}
