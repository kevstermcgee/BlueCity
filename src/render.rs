//! Read-only, asset-free isometric presentation for the BlueCity simulation.
use crate::city::{City, Kind, Terrain, Tile, SIZE};
use macroquad::prelude::*;
use vesper3d::two_d::{draw::Scene, Rect as LogicalRect};

#[derive(Clone, Copy)]
pub struct View {
    pub cam_x: i32,
    pub cam_y: i32,
    pub zoom: i32,
    pub overlay: u8,
    pub hover: Option<[i32; 2]>,
    pub anchor: Option<[i32; 2]>,
    pub tick: u32,
}

pub fn project(x: i32, y: i32, view: View) -> (i32, i32) {
    let w = match view.zoom {
        1 => 8,
        3 => 18,
        _ => 12,
    };
    (
        296 + (x - view.cam_x - y + view.cam_y) * w,
        217 + (x - view.cam_x + y - view.cam_y) * (w / 2),
    )
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}
fn tint(c: Color, v: f32) -> Color {
    Color::new((c.r * v).min(1.), (c.g * v).min(1.), (c.b * v).min(1.), c.a)
}
fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}
fn mix(a: Color, b: Color, f: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * f,
        a.g + (b.g - a.g) * f,
        a.b + (b.b - a.b) * f,
        a.a + (b.a - a.a) * f,
    )
}
fn quad(a: Vec2, b: Vec2, c: Vec2, d: Vec2, col: Color) {
    draw_triangle(a, b, c, col);
    draw_triangle(a, c, d, col);
}
fn noise(x: i32, y: i32) -> u32 {
    let mut n = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}

#[derive(Clone, Copy)]
struct Iso {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
impl Iso {
    fn p(self, u: f32, v: f32, z: f32) -> Vec2 {
        vec2(self.x + (u - v) * self.w, self.y + (u + v) * self.h - z)
    }
    fn diamond(self, c: Color) {
        quad(
            self.p(-0.5, -0.5, 0.),
            self.p(0.5, -0.5, 0.),
            self.p(0.5, 0.5, 0.),
            self.p(-0.5, 0.5, 0.),
            c,
        );
    }
    fn patch(self, u: f32, v: f32, w: f32, h: f32, z: f32, c: Color) {
        quad(
            self.p(u - w / 2., v - h / 2., z),
            self.p(u + w / 2., v - h / 2., z),
            self.p(u + w / 2., v + h / 2., z),
            self.p(u - w / 2., v + h / 2., z),
            c,
        );
    }
    // Explicit coordinates mirror the drawing API and keep authored geometry readable.
    #[allow(clippy::too_many_arguments)]
    fn line(self, u: f32, v: f32, u2: f32, v2: f32, z: f32, width: f32, c: Color) {
        let a = self.p(u, v, z);
        let b = self.p(u2, v2, z);
        draw_line(a.x, a.y, b.x, b.y, width, c);
    }
    fn edge(self, c: Color, width: f32) {
        let a = self.p(-0.5, -0.5, 0.);
        let b = self.p(0.5, -0.5, 0.);
        let d = self.p(-0.5, 0.5, 0.);
        let e = self.p(0.5, 0.5, 0.);
        for (p, q) in [(a, b), (b, e), (e, d), (d, a)] {
            draw_line(p.x, p.y, q.x, q.y, width, c);
        }
    }
    fn block(self, u: f32, v: f32, w: f32, d: f32, z: f32, c: Color) {
        let a = self.p(u - w / 2., v - d / 2., 0.);
        let b = self.p(u + w / 2., v - d / 2., 0.);
        let e = self.p(u + w / 2., v + d / 2., 0.);
        let f = self.p(u - w / 2., v + d / 2., 0.);
        let lift = vec2(0., z);
        quad(b, e, e - lift, b - lift, tint(c, 0.67));
        quad(e, f, f - lift, e - lift, tint(c, 0.83));
        quad(a - lift, b - lift, e - lift, f - lift, c);
    }
    // Footprint, elevations and color describe one authored building primitive.
    #[allow(clippy::too_many_arguments)]
    fn raised(self, u: f32, v: f32, w: f32, d: f32, base: f32, height: f32, c: Color) {
        Self {
            y: self.y - base,
            ..self
        }
        .block(u, v, w, d, height, c);
    }
    // Window placement shares the building footprint plus its visual seed and color.
    #[allow(clippy::too_many_arguments)]
    fn windows(self, u: f32, v: f32, w: f32, d: f32, z: f32, seed: u32, glass: Color) {
        let scale = self.w / 12.;
        let rows = (z / (4. * scale)).floor() as u32;
        let cols = if w > 0.65 { 3 } else { 2 };
        for row in 0..rows.saturating_sub(1) {
            for col in 0..cols {
                let h = 2.5 * scale + row as f32 * 4. * scale;
                let pos = (col as f32 + 0.5) / cols as f32;
                let c = if (seed + row * 5 + col * 3).is_multiple_of(7) {
                    rgb(255, 223, 138)
                } else {
                    glass
                };
                let a = self.p(u - w / 2. + w * pos, v + d / 2. + 0.006, h);
                let b = self.p(u + w / 2. + 0.006, v - d / 2. + d * pos, h);
                draw_line(
                    a.x - 0.7 * scale,
                    a.y - 0.35 * scale,
                    a.x + 0.7 * scale,
                    a.y + 0.35 * scale,
                    1.05 * scale,
                    c,
                );
                draw_line(
                    b.x - 0.7 * scale,
                    b.y + 0.35 * scale,
                    b.x + 0.7 * scale,
                    b.y - 0.35 * scale,
                    1.05 * scale,
                    tint(c, 0.86),
                );
            }
        }
    }
    fn shadow(self, u: f32, v: f32, w: f32, d: f32, z: f32) {
        let c = Color::new(0.07, 0.16, 0.13, 0.16);
        quad(
            self.p(u - w / 2., v - d / 2., 0.),
            self.p(u + w / 2., v - d / 2., 0.),
            self.p(u + w / 2. + z / 55., v + d / 2. + z / 90., 0.),
            self.p(u - w / 2. + z / 55., v + d / 2. + z / 90., 0.),
            c,
        );
    }
}

pub fn map<'a>(city: &'a City, view: View, scene: &mut Scene<'a>) {
    scene.render_view(0, LogicalRect::new(0, 64, 592, 306), move |viewport| {
        let mut camera =
            Camera2D::from_display_rect(macroquad::prelude::Rect::new(0., 64., 592., 306.));
        camera.viewport = Some(viewport);
        set_camera(&camera);
        draw_rectangle(0., 64., 592., 306., rgb(18, 38, 44));
        // The grid in the sea of the world makes the edge of the buildable region clear.
        for n in -35..60 {
            let a = project(n, 0, view);
            let b = project(n, SIZE as i32, view);
            draw_line(
                a.0 as f32,
                a.1 as f32,
                b.0 as f32,
                b.1 as f32,
                0.4,
                rgb(24, 47, 51),
            );
            let a = project(0, n, view);
            let b = project(SIZE as i32, n, view);
            draw_line(
                a.0 as f32,
                a.1 as f32,
                b.0 as f32,
                b.1 as f32,
                0.4,
                rgb(24, 47, 51),
            );
        }
        let w = match view.zoom {
            1 => 8.,
            3 => 18.,
            _ => 12.,
        };
        // Back-to-front diagonal order also orders trees, cars, and building facades.
        for sum in 0..(SIZE * 2 - 1) {
            for y in 0..SIZE {
                if sum < y {
                    continue;
                }
                let x = sum - y;
                if x >= SIZE {
                    continue;
                }
                let (sx, sy) = project(x as i32, y as i32, view);
                if sx < -(w as i32) - 8 || sx > 610 || !(40..=450).contains(&sy) {
                    continue;
                }
                let iso = Iso {
                    x: sx as f32,
                    y: sy as f32,
                    w,
                    h: w / 2.,
                };
                let t = &city.tiles[y * SIZE + x];
                let n = noise(x as i32, y as i32);
                ground(city, t, iso, x, y, n, view);
                object(city, t, iso, x, y, n, view);
                if view.overlay > 0 {
                    overlay(t, iso, view.overlay);
                }
                if let (Some(a), Some(h)) = (view.anchor, view.hover) {
                    let tx = x as i32;
                    let ty = y as i32;
                    if tx >= a[0].min(h[0])
                        && tx <= a[0].max(h[0])
                        && ty >= a[1].min(h[1])
                        && ty <= a[1].max(h[1])
                    {
                        iso.diamond(Color::new(0.22, 0.95, 0.84, 0.2));
                        iso.edge(Color::new(0.4, 1., 0.9, 0.6), 0.8);
                    }
                }
                if t.fire > 0 {
                    fire(iso, t.fire, view.tick, n);
                }
            }
        }
        if let Some([x, y]) = view.hover {
            if x >= 0 && y >= 0 && x < SIZE as i32 && y < SIZE as i32 {
                let (sx, sy) = project(x, y, view);
                let iso = Iso {
                    x: sx as f32,
                    y: sy as f32,
                    w,
                    h: w / 2.,
                };
                iso.diamond(Color::new(0.62, 1., 0.88, 0.16));
                iso.edge(rgb(164, 255, 222), 1.5);
                let p = iso.p(-0.5, -0.5, 0.);
                draw_circle(p.x, p.y, 1.5, rgb(204, 255, 234));
            }
        }
        // A subtle vignette edge anchors the map without obscuring construction.
        draw_rectangle(0., 64., 592., 1., rgb(49, 80, 82));
        draw_rectangle(0., 369., 592., 1., rgb(8, 24, 31));
        set_default_camera();
        Ok(())
    });
}

fn neighbour(city: &City, x: usize, y: usize, dx: i32, dy: i32) -> Option<&Tile> {
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 || nx >= SIZE as i32 || ny >= SIZE as i32 {
        None
    } else {
        Some(&city.tiles[ny as usize * SIZE + nx as usize])
    }
}
fn street(k: Kind) -> bool {
    matches!(k, Kind::Road | Kind::Avenue)
}
fn ground(city: &City, t: &Tile, i: Iso, x: usize, y: usize, n: u32, v: View) {
    let grass = match n % 5 {
        0 => rgb(119, 156, 105),
        1 => rgb(125, 163, 109),
        2 => rgb(129, 166, 113),
        3 => rgb(124, 160, 107),
        _ => rgb(132, 166, 114),
    };
    match t.terrain {
        Terrain::Water => {
            let c = match n % 4 {
                0 => rgb(48, 124, 148),
                1 => rgb(45, 119, 144),
                _ => rgb(49, 127, 149),
            };
            i.diamond(c);
            for (dx, dy, u1, v1, u2, v2) in [
                (1, 0, 0.5, -0.5, 0.5, 0.5),
                (0, 1, -0.5, 0.5, 0.5, 0.5),
                (-1, 0, -0.5, -0.5, -0.5, 0.5),
                (0, -1, -0.5, -0.5, 0.5, -0.5),
            ] {
                if neighbour(city, x, y, dx, dy).is_some_and(|t| t.terrain != Terrain::Water) {
                    i.line(u1, v1, u2, v2, 0., 1.4, rgb(122, 184, 172));
                }
            }
            if (n + v.tick / 20) % 7 < 2 {
                let u = ((n >> 3) % 5) as f32 / 10. - 0.2;
                i.line(-0.23, u, 0.1, u, 0., 0.7, alpha(rgb(154, 216, 213), 0.4));
            }
        }
        Terrain::Hill => {
            i.diamond(tint(grass, 0.9));
            let top = i.p(-0.1, -0.1, 3. * i.w / 12.);
            draw_triangle(
                i.p(-0.5, -0.5, 0.),
                i.p(0.5, -0.5, 0.),
                top,
                tint(grass, 1.06),
            );
            draw_triangle(
                i.p(0.5, -0.5, 0.),
                i.p(0.5, 0.5, 0.),
                top,
                tint(grass, 0.89),
            );
            draw_triangle(i.p(0.5, 0.5, 0.), i.p(-0.5, 0.5, 0.), top, grass);
            draw_triangle(
                i.p(-0.5, 0.5, 0.),
                i.p(-0.5, -0.5, 0.),
                top,
                tint(grass, 1.03),
            );
        }
        _ => {
            i.diamond(grass);
            if n.is_multiple_of(6) {
                i.patch(0.1, -0.2, 0.08, 0.12, 0., tint(grass, 1.09));
            }
        }
    }
    if matches!(
        t.kind,
        Kind::Residential | Kind::Commercial | Kind::Industrial
    ) {
        let c = match t.kind {
            Kind::Residential => rgb(103, 178, 126),
            Kind::Commercial => rgb(93, 164, 195),
            _ => rgb(208, 175, 95),
        };
        if t.level == 0 {
            i.diamond(mix(grass, c, 0.42));
            i.edge(alpha(tint(c, 1.25), 0.65), 0.55);
            i.patch(0., 0., 0.22, 0.22, 0., alpha(c, 0.7));
        } else {
            i.diamond(mix(grass, rgb(164, 167, 151), 0.25));
        }
    }
    // Map edges sit on a shallow, cutaway plinth.
    if x == SIZE - 1 {
        quad(
            i.p(0.5, -0.5, 0.),
            i.p(0.5, 0.5, 0.),
            i.p(0.5, 0.5, -4.),
            i.p(0.5, -0.5, -4.),
            rgb(62, 89, 68),
        );
    }
    if y == SIZE - 1 {
        quad(
            i.p(-0.5, 0.5, 0.),
            i.p(0.5, 0.5, 0.),
            i.p(0.5, 0.5, -4.),
            i.p(-0.5, 0.5, -4.),
            rgb(74, 101, 73),
        );
    }
}

fn tree(i: Iso, u: f32, v: f32, n: u32, size: f32) {
    let s = i.w / 12. * size;
    let p = i.p(u, v, 0.);
    draw_ellipse(
        p.x + 1.8 * s,
        p.y + 0.6 * s,
        3.1 * s,
        1.2 * s,
        0.,
        Color::new(0.08, 0.19, 0.12, 0.16),
    );
    draw_line(p.x, p.y, p.x, p.y - 5. * s, 1.3 * s, rgb(101, 95, 65));
    let c = match n % 4 {
        0 => rgb(40, 112, 83),
        1 => rgb(54, 131, 87),
        2 => rgb(67, 143, 91),
        _ => rgb(79, 147, 94),
    };
    draw_circle(p.x, p.y - 6.5 * s, 3.9 * s, c);
    draw_circle(p.x - 1.1 * s, p.y - 8. * s, 2.7 * s, tint(c, 1.12));
    draw_circle(p.x + 1.7 * s, p.y - 6. * s, 2.3 * s, tint(c, 0.91));
}
fn conifer(i: Iso, u: f32, v: f32, n: u32) {
    let p = i.p(u, v, 0.);
    let s = i.w / 12.;
    draw_line(p.x, p.y, p.x, p.y - 5. * s, 1. * s, rgb(99, 88, 61));
    for b in 0..3 {
        let h = (b as f32) * 2.2 * s;
        let w = (3.8 - b as f32 * 0.7) * s;
        draw_triangle(
            vec2(p.x - w, p.y - 2. * s - h),
            vec2(p.x + w, p.y - 2. * s - h),
            vec2(p.x, p.y - 8. * s - h),
            if n.is_multiple_of(2) {
                rgb(42, 112, 80)
            } else {
                rgb(51, 125, 83)
            },
        );
    }
}

fn object(city: &City, t: &Tile, i: Iso, x: usize, y: usize, n: u32, v: View) {
    let s = i.w / 12.;
    match t.kind {
        Kind::Empty => {
            if t.terrain == Terrain::Forest {
                tree(i, -0.2, -0.22, n, 1.);
                if !n.is_multiple_of(3) {
                    tree(i, 0.22, 0.1, n + 1, 0.8);
                }
                if n.is_multiple_of(3) {
                    conifer(i, 0.2, 0.2, n);
                }
            } else if t.terrain == Terrain::Hill && n.is_multiple_of(3) {
                conifer(i, 0.1, 0., n);
            }
        }
        Kind::Road | Kind::Avenue => road(city, t, i, x, y, n, v),
        Kind::Rail => rail(city, i, x, y, v),
        Kind::PowerLine => {
            let p = i.p(0., 0., 0.);
            let h = 13. * s;
            draw_line(p.x - 2. * s, p.y, p.x, p.y - h, 0.9 * s, rgb(75, 87, 81));
            draw_line(p.x + 2. * s, p.y, p.x, p.y - h, 0.9 * s, rgb(75, 87, 81));
            draw_line(
                p.x - 3. * s,
                p.y - h + 2. * s,
                p.x + 3. * s,
                p.y - h + 2. * s,
                1.1 * s,
                rgb(188, 189, 165),
            );
            for (dx, dy) in [(1, 0), (0, 1)] {
                if neighbour(city, x, y, dx, dy).is_some_and(|t| {
                    matches!(
                        t.kind,
                        Kind::PowerLine | Kind::Coal | Kind::Wind | Kind::Nuclear
                    )
                }) {
                    let p2 = i.p(dx as f32, dy as f32, h - 2. * s);
                    let mid = (p + vec2(0., -h + 2. * s) + p2) / 2. + vec2(0., 2. * s);
                    draw_line(
                        p.x,
                        p.y - h + 2. * s,
                        mid.x,
                        mid.y,
                        0.6 * s,
                        rgb(56, 76, 70),
                    );
                    draw_line(mid.x, mid.y, p2.x, p2.y, 0.6 * s, rgb(56, 76, 70));
                }
            }
        }
        Kind::Pipe => {
            i.patch(0., 0., 0.85, 0.22, 0., rgb(102, 155, 156));
            i.line(-0.5, 0., 0.5, 0., 1. * s, 2. * s, rgb(119, 206, 216));
            i.block(0., 0., 0.18, 0.25, 2. * s, rgb(182, 215, 199));
        }
        Kind::Residential => {
            if t.level > 0 {
                residence(i, t, n);
            }
        }
        Kind::Commercial => {
            if t.level > 0 {
                commercial(i, t, n);
            }
        }
        Kind::Industrial => {
            if t.level > 0 {
                industry(i, t, n, v.tick);
            }
        }
        Kind::Park => {
            i.diamond(rgb(105, 164, 108));
            i.patch(0., 0., 0.16, 1., 0., rgb(207, 195, 153));
            i.patch(0., 0., 1., 0.14, 0., rgb(207, 195, 153));
            tree(i, -0.25, -0.2, n, 0.9);
            tree(i, 0.25, 0.22, n + 1, 0.8);
            let p = i.p(0., 0., 0.);
            draw_ellipse(p.x, p.y, 2.3 * s, 1.2 * s, 0., rgb(130, 206, 212));
            i.line(-0.35, 0.28, -0.15, 0.28, 1. * s, 1.3 * s, rgb(139, 107, 70));
        }
        Kind::Wind => wind(i, v.tick, n),
        Kind::Coal => {
            i.diamond(rgb(139, 151, 137));
            i.block(-0.08, 0.1, 0.76, 0.64, 9. * s, rgb(164, 158, 128));
            i.patch(-0.08, 0.1, 0.58, 0.44, 9.1 * s, rgb(94, 115, 116));
            for (u, h) in [(-0.24, 24.), (0.18, 20.)] {
                i.block(u, -0.28, 0.14, 0.16, h * s, rgb(171, 158, 141));
                i.raised(u, -0.28, 0.16, 0.18, h * s, s, rgb(93, 105, 105));
                smoke(i, u, -0.28, (h + 2.) * s, v.tick, n, 0.8);
            }
        }
        Kind::Nuclear => {
            i.diamond(rgb(151, 162, 147));
            i.block(-0.2, 0.18, 0.5, 0.45, 8. * s, rgb(176, 196, 184));
            for (u, z) in [(0.18, -0.12), (-0.18, -0.26)] {
                let p = i.p(u, z, 0.);
                let a = vec2(p.x - 3. * s, p.y);
                let b = vec2(p.x + 3. * s, p.y);
                quad(
                    a,
                    b,
                    b + vec2(-0.8 * s, -13. * s),
                    a + vec2(0.8 * s, -13. * s),
                    rgb(181, 196, 189),
                );
                draw_ellipse(p.x, p.y - 13. * s, 2.3 * s, 1.1 * s, 0., rgb(107, 138, 137));
                smoke(i, u, z, 15. * s, v.tick, n, 0.55);
            }
        }
        Kind::Pump => {
            i.diamond(rgb(134, 171, 161));
            i.block(-0.13, 0.1, 0.62, 0.55, 6. * s, rgb(194, 206, 179));
            let p = i.p(0.2, -0.15, 0.);
            draw_line(p.x, p.y, p.x, p.y - 13. * s, 1.5 * s, rgb(122, 150, 144));
            draw_ellipse(p.x, p.y - 12. * s, 3.4 * s, 2.6 * s, 0., rgb(143, 199, 202));
            draw_ellipse(p.x, p.y - 14. * s, 3.4 * s, 1.6 * s, 0., rgb(197, 233, 222));
        }
        Kind::Sewage => {
            i.diamond(rgb(131, 163, 145));
            i.block(-0.26, -0.1, 0.36, 0.54, 6. * s, rgb(165, 185, 158));
            for (u, z) in [(0.15, -0.22), (0.15, 0.2)] {
                i.patch(u, z, 0.4, 0.3, 1. * s, rgb(218, 218, 187));
                i.patch(u, z, 0.3, 0.22, 1.2 * s, rgb(83, 132, 132));
            }
        }
        Kind::Police => service(i, n, rgb(101, 145, 167), rgb(182, 220, 222), 2),
        Kind::Fire => {
            service(i, n, rgb(198, 113, 91), rgb(240, 188, 126), 1);
            for u in [-0.2, 0.12] {
                i.block(u, 0.28, 0.22, 0.23, 3. * s, rgb(218, 85, 66));
            }
        }
        Kind::Hospital => {
            service(i, n, rgb(224, 226, 205), rgb(112, 174, 173), 0);
            let p = i.p(0., 0., 12.3 * s);
            draw_rectangle(p.x - 1. * s, p.y - 3. * s, 2. * s, 6. * s, rgb(218, 91, 86));
            draw_rectangle(p.x - 3. * s, p.y - 1. * s, 6. * s, 2. * s, rgb(218, 91, 86));
        }
        Kind::School => {
            service(i, n, rgb(203, 171, 123), rgb(110, 161, 159), 3);
            i.patch(0.25, 0.23, 0.27, 0.3, 0., rgb(140, 181, 160));
            let p = i.p(-0.3, 0.28, 0.);
            draw_line(p.x, p.y, p.x, p.y - 12. * s, 0.7 * s, rgb(200, 214, 194));
            draw_triangle(
                vec2(p.x, p.y - 12. * s),
                vec2(p.x + 4. * s, p.y - 11. * s),
                vec2(p.x, p.y - 9. * s),
                rgb(104, 192, 209),
            );
        }
        Kind::University => {
            i.diamond(rgb(153, 174, 135));
            i.block(0., 0., 0.86, 0.68, 9. * s, rgb(211, 193, 158));
            i.raised(0., -0.12, 0.3, 0.36, 9. * s, 10. * s, rgb(200, 183, 151));
            i.patch(0., -0.12, 0.34, 0.4, 19.2 * s, rgb(79, 133, 141));
            i.windows(0., 0., 0.86, 0.68, 9. * s, n, rgb(103, 146, 145));
            let p = i.p(0., -0.12, 17. * s);
            draw_circle(p.x, p.y, 1.5 * s, rgb(248, 232, 180));
        }
        Kind::Stadium => stadium(i),
        Kind::BusDepot => {
            service(i, n, rgb(130, 179, 167), rgb(65, 123, 141), 3);
            for u in [-0.23, 0.12] {
                i.block(u, 0.26, 0.18, 0.42, 3. * s, rgb(236, 190, 92));
                i.patch(u, 0.26, 0.1, 0.3, 3.2 * s, rgb(87, 133, 137));
            }
        }
        Kind::Station => {
            i.diamond(rgb(168, 175, 153));
            i.block(-0.1, -0.05, 0.7, 0.66, 6. * s, rgb(211, 198, 155));
            i.patch(-0.1, -0.05, 0.75, 0.7, 6.2 * s, rgb(82, 147, 155));
            i.line(0.3, -0.5, 0.3, 0.5, 0., 1.2 * s, rgb(67, 89, 87));
            i.line(0.42, -0.5, 0.42, 0.5, 0., 1.2 * s, rgb(67, 89, 87));
        }
        Kind::Seaport => {
            i.diamond(rgb(155, 172, 158));
            i.block(-0.2, 0.05, 0.4, 0.72, 7. * s, rgb(143, 165, 149));
            for (u, c) in [(0.12, rgb(199, 130, 91)), (0.34, rgb(93, 159, 168))] {
                i.block(u, 0.1, 0.18, 0.64, 4. * s, c);
            }
            let a = i.p(0.2, -0.3, 0.);
            let b = i.p(0.2, -0.3, 20. * s);
            draw_line(a.x, a.y, b.x, b.y, 1.5 * s, rgb(226, 190, 101));
            draw_line(
                b.x - 6. * s,
                b.y,
                b.x + 8. * s,
                b.y,
                1.5 * s,
                rgb(226, 190, 101),
            );
            draw_line(
                b.x + 7. * s,
                b.y,
                b.x + 7. * s,
                b.y + 8. * s,
                0.7 * s,
                rgb(79, 109, 110),
            );
        }
        Kind::Airport => airport(i, v.tick, n),
        Kind::Landfill => {
            i.diamond(rgb(149, 147, 111));
            for k in 0..5 {
                let u = (k % 3) as f32 * 0.25 - 0.25;
                let z = (k / 3) as f32 * 0.35 - 0.17;
                i.block(
                    u,
                    z,
                    0.28,
                    0.28,
                    (3. + ((n >> k) % 4) as f32) * s,
                    rgb(151 + (k * 7) as u8, 143, 111),
                );
            }
            i.block(0.28, 0.24, 0.25, 0.35, 4. * s, rgb(210, 174, 86));
        }
    }
}

fn road(city: &City, t: &Tile, i: Iso, x: usize, y: usize, n: u32, v: View) {
    let s = i.w / 12.;
    let wide = t.kind == Kind::Avenue;
    let r = if wide { 0.43 } else { 0.32 };
    i.diamond(if t.terrain == Terrain::Water {
        rgb(108, 134, 129)
    } else {
        rgb(169, 178, 151)
    });
    let asphalt = if wide {
        rgb(77, 100, 105)
    } else {
        rgb(88, 110, 111)
    };
    i.patch(0., 0., r * 2., r * 2., 0., asphalt);
    let mut joins = 0;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let connected = neighbour(city, x, y, dx, dy).is_some_and(|t| street(t.kind));
        if connected {
            joins += 1;
            if dx != 0 {
                i.patch(dx as f32 * (0.5 + r) / 2., 0., 0.5 - r, r * 2., 0., asphalt);
            } else {
                i.patch(0., dy as f32 * (0.5 + r) / 2., r * 2., 0.5 - r, 0., asphalt);
            }
            let d = 0.24;
            i.line(
                dx as f32 * d,
                dy as f32 * d,
                dx as f32 * 0.45,
                dy as f32 * 0.45,
                0.,
                0.65 * s,
                if wide {
                    rgb(226, 200, 131)
                } else {
                    rgb(201, 208, 183)
                },
            );
        }
    }
    if wide {
        i.patch(0., 0., 0.12, 0.12, 0., rgb(135, 160, 124));
    }
    if joins >= 3 && v.zoom >= 2 {
        for k in 0..3 {
            let a = -0.19 + k as f32 * 0.15;
            i.line(a, 0.28, a + 0.07, 0.28, 0., 0.8 * s, rgb(211, 220, 195));
        }
    }
    if t.rail_crossing {
        track_slats(i, rail_axis_x(city, x, y));
        let signal = i.p(0.34, 0.34, 0.);
        draw_line(
            signal.x,
            signal.y,
            signal.x,
            signal.y - 5. * s,
            0.7 * s,
            rgb(192, 205, 180),
        );
        draw_circle(signal.x, signal.y - 5. * s, 1. * s, rgb(232, 132, 89));
    }
    if t.terrain == Terrain::Water {
        for u in [-0.4, 0.4] {
            let p = i.p(u, 0.4, 0.);
            draw_line(p.x, p.y, p.x, p.y + 4. * s, 1.2 * s, rgb(87, 111, 109));
        }
    }
    if t.traffic > 0 || n.is_multiple_of(5) {
        let along_x = neighbour(city, x, y, 1, 0).is_some_and(|t| street(t.kind));
        let t = ((v.tick + n % 140) % 140) as f32 / 140. - 0.5;
        let (u, z) = if along_x { (t, 0.13) } else { (-0.13, t) };
        let color = match n % 5 {
            0 => rgb(245, 209, 119),
            1 => rgb(209, 111, 96),
            2 => rgb(213, 225, 211),
            3 => rgb(106, 181, 195),
            _ => rgb(130, 151, 184),
        };
        i.block(
            u,
            z,
            if along_x { 0.22 } else { 0.12 },
            if along_x { 0.12 } else { 0.22 },
            1.6 * s,
            color,
        );
        if t > 0.05 && t < 0.3 {
            let p = i.p(u, z, 1.8 * s);
            draw_rectangle(
                p.x - 0.7 * s,
                p.y - 0.4 * s,
                1.3 * s,
                0.7 * s,
                rgb(83, 124, 133),
            );
        }
    }
}

fn rail_axis_x(city: &City, x: usize, y: usize) -> bool {
    [-1, 1].into_iter().any(|dx| {
        neighbour(city, x, y, dx, 0)
            .is_some_and(|t| matches!(t.kind, Kind::Rail | Kind::Station) || t.rail_crossing)
    })
}

fn track_slats(i: Iso, along_x: bool) {
    let s = i.w / 12.;
    for k in 0..5 {
        let p = -0.42 + k as f32 * 0.21;
        if along_x {
            i.line(p, -0.24, p, 0.24, 0., 1.1 * s, rgb(114, 107, 83));
        } else {
            i.line(-0.24, p, 0.24, p, 0., 1.1 * s, rgb(114, 107, 83));
        }
    }
    for z in [-0.15, 0.15] {
        if along_x {
            i.line(-0.5, z, 0.5, z, 0., 0.8 * s, rgb(217, 218, 190));
        } else {
            i.line(z, -0.5, z, 0.5, 0., 0.8 * s, rgb(217, 218, 190));
        }
    }
}

fn rail(city: &City, i: Iso, x: usize, y: usize, v: View) {
    let s = i.w / 12.;
    let along_x = rail_axis_x(city, x, y);
    i.patch(
        0.,
        0.,
        if along_x { 1. } else { 0.52 },
        if along_x { 0.52 } else { 1. },
        0.,
        rgb(146, 148, 126),
    );
    track_slats(i, along_x);
    if (v.tick / 12 + x as u32 + y as u32) % 19 < 3 {
        i.block(
            0.,
            0.,
            if along_x { 0.72 } else { 0.26 },
            if along_x { 0.26 } else { 0.72 },
            3. * s,
            rgb(207, 170, 101),
        );
        i.patch(
            0.,
            0.,
            if along_x { 0.64 } else { 0.18 },
            if along_x { 0.18 } else { 0.64 },
            3.2 * s,
            rgb(86, 137, 143),
        );
    }
}

fn residence(i: Iso, t: &Tile, n: u32) {
    let s = i.w / 12.;
    let c = match n % 5 {
        0 => rgb(225, 206, 162),
        1 => rgb(232, 219, 180),
        2 => rgb(190, 208, 174),
        3 => rgb(214, 191, 157),
        _ => rgb(213, 222, 184),
    };
    if t.level == 1 {
        let u = if n.is_multiple_of(2) { -0.06 } else { 0.08 };
        let z = -0.07;
        i.patch(0.23, 0.27, 0.15, 0.43, 0., rgb(202, 198, 161));
        i.shadow(u, z, 0.62, 0.6, 7. * s);
        i.block(u, z, 0.62, 0.6, 6. * s, c);
        // A pitched terracotta roof and its two gables distinguish low-density homes.
        let a = i.p(u - 0.36, z - 0.35, 6. * s);
        let b = i.p(u + 0.36, z - 0.35, 6. * s);
        let d = i.p(u - 0.36, z + 0.35, 6. * s);
        let e = i.p(u + 0.36, z + 0.35, 6. * s);
        let r1 = i.p(u - 0.36, z, 10. * s);
        let r2 = i.p(u + 0.36, z, 10. * s);
        quad(
            a,
            b,
            r2,
            r1,
            if n.is_multiple_of(3) {
                rgb(84, 136, 142)
            } else {
                rgb(188, 122, 91)
            },
        );
        quad(
            r1,
            r2,
            e,
            d,
            if n.is_multiple_of(3) {
                rgb(64, 113, 126)
            } else {
                rgb(151, 100, 81)
            },
        );
        draw_triangle(b, e, r2, tint(c, 0.73));
        i.windows(u, z, 0.62, 0.6, 6. * s, n, rgb(103, 158, 162));
        i.raised(
            u - 0.16,
            z - 0.1,
            0.09,
            0.1,
            7. * s,
            4. * s,
            rgb(160, 133, 108),
        );
        if n.is_multiple_of(2) {
            tree(i, -0.28, 0.28, n, 0.6);
        }
    } else {
        let h = (9. + t.level as f32 * 4. + (n % 3) as f32 * 2.) * s;
        i.shadow(0., 0., 0.69, 0.68, h);
        i.block(0., 0., 0.69, 0.68, h, c);
        i.windows(0., 0., 0.69, 0.68, h, n, rgb(111, 159, 161));
        i.patch(0., 0., 0.6, 0.59, h + 0.2 * s, rgb(132, 166, 159));
        i.raised(-0.15, -0.14, 0.19, 0.21, h, 2. * s, rgb(192, 205, 182));
        for k in 0..t.level.min(4) {
            let h = (5. + k as f32 * 5.) * s;
            i.line(-0.35, 0.35, 0.35, 0.35, h, 0.8 * s, tint(c, 0.9));
        }
    }
}

fn commercial(i: Iso, t: &Tile, n: u32) {
    let s = i.w / 12.;
    let h = if t.level == 1 {
        6. * s
    } else {
        (10. + t.level as f32 * 7. + (n % 4) as f32 * 2.) * s
    };
    let c = match n % 4 {
        0 => rgb(137, 190, 190),
        1 => rgb(136, 175, 185),
        2 => rgb(156, 196, 196),
        _ => rgb(153, 183, 188),
    };
    i.shadow(0., 0., 0.75, 0.72, h);
    i.block(0., 0., 0.75, 0.72, h, c);
    i.windows(0., 0., 0.75, 0.72, h, n, rgb(191, 223, 212));
    if t.level == 1 {
        let c = if n.is_multiple_of(2) {
            rgb(236, 181, 101)
        } else {
            rgb(116, 181, 172)
        };
        i.line(-0.4, 0.38, 0.4, 0.38, 4. * s, 2. * s, c);
        i.patch(0., 0., 0.7, 0.67, h + 0.2 * s, rgb(116, 155, 159));
    } else {
        i.patch(0., 0., 0.68, 0.65, h + 0.2 * s, rgb(98, 151, 163));
        i.raised(-0.13, -0.15, 0.26, 0.24, h, 3. * s, rgb(153, 188, 185));
        if t.level >= 4 {
            let p = i.p(-0.13, -0.15, h + 3. * s);
            draw_line(p.x, p.y, p.x, p.y - 8. * s, 0.6 * s, rgb(192, 219, 199));
            draw_circle(p.x, p.y - 8. * s, 0.8 * s, rgb(247, 207, 119));
        }
    }
}

fn industry(i: Iso, t: &Tile, n: u32, tick: u32) {
    let s = i.w / 12.;
    let h = (5. + t.level as f32 * 1.6) * s;
    i.diamond(rgb(169, 169, 133));
    i.shadow(0., 0.02, 0.86, 0.7, h);
    i.block(0., 0.02, 0.86, 0.7, h, rgb(196, 173, 119));
    // Sawtooth roof strips and warm warehouse doors remain readable at every zoom.
    for k in 0..3 {
        let u = -0.27 + k as f32 * 0.27;
        i.patch(
            u,
            0.02,
            0.2,
            0.65,
            h + 0.2 * s,
            if k % 2 == 0 {
                rgb(142, 159, 145)
            } else {
                rgb(176, 188, 163)
            },
        );
    }
    i.windows(0., 0.02, 0.86, 0.7, h, n, rgb(127, 151, 143));
    i.block(
        0.28,
        -0.23,
        0.14,
        0.14,
        (15. + t.level as f32 * 2.) * s,
        rgb(173, 143, 111),
    );
    smoke(
        i,
        0.28,
        -0.23,
        (16. + t.level as f32 * 2.) * s,
        tick,
        n,
        0.62,
    );
    i.block(-0.25, 0.29, 0.16, 0.17, 2. * s, rgb(202, 153, 79));
}

fn service(i: Iso, n: u32, c: Color, roof: Color, style: u8) {
    let s = i.w / 12.;
    i.diamond(rgb(164, 182, 151));
    let h = if style == 0 { 12. } else { 8. } * s;
    i.shadow(0., -0.08, 0.82, 0.64, h);
    i.block(0., -0.08, 0.82, 0.64, h, c);
    i.patch(0., -0.08, 0.75, 0.57, h + 0.2 * s, roof);
    i.windows(0., -0.08, 0.82, 0.64, h, n, rgb(109, 162, 163));
    i.patch(0.1, 0.34, 0.7, 0.18, 0., rgb(190, 194, 164));
    if style == 2 {
        let p = i.p(0., 0., h + 2. * s);
        draw_poly(p.x, p.y, 5, 2.6 * s, -90., rgb(240, 212, 131));
    }
    if style == 1 {
        i.raised(-0.28, -0.2, 0.2, 0.2, 8. * s, 7. * s, tint(c, 0.95));
    }
}

fn smoke(i: Iso, u: f32, v: f32, h: f32, tick: u32, n: u32, strength: f32) {
    let s = i.w / 12.;
    let phase = (tick + n % 100) % 100;
    for k in 0..3 {
        let age = ((phase + k * 33) % 100) as f32 / 100.;
        let p = i.p(u, v, h);
        let r = (1. + age * 3.) * s;
        draw_circle(
            p.x + age * 4. * s,
            p.y - age * 12. * s,
            r,
            Color::new(0.73, 0.78, 0.71, (1. - age) * 0.22 * strength),
        );
    }
}
fn wind(i: Iso, tick: u32, n: u32) {
    let s = i.w / 12.;
    let p = i.p(0., 0., 0.);
    draw_ellipse(
        p.x + 2. * s,
        p.y + 0.5 * s,
        3.5 * s,
        1.1 * s,
        0.,
        Color::new(0.1, 0.2, 0.16, 0.18),
    );
    draw_line(p.x, p.y, p.x, p.y - 22. * s, 1.6 * s, rgb(206, 224, 206));
    let center = vec2(p.x, p.y - 22. * s);
    let a = (tick % 720) as f32 * 0.022 + (n % 360) as f32 * 0.0174533;
    for k in 0..3 {
        let r = a + k as f32 * std::f32::consts::TAU / 3.;
        let tip = center + vec2(r.cos(), r.sin()) * 8.2 * s;
        let mid = center + vec2((r + 0.2).cos(), (r + 0.2).sin()) * 3.5 * s;
        draw_triangle(center, tip, mid, rgb(235, 238, 214));
    }
    draw_circle(center.x, center.y, 1.2 * s, rgb(173, 204, 193));
}
fn stadium(i: Iso) {
    let s = i.w / 12.;
    i.diamond(rgb(161, 184, 143));
    i.block(0., 0., 0.9, 0.86, 5. * s, rgb(198, 193, 163));
    i.patch(0., 0., 0.65, 0.6, 5.2 * s, rgb(61, 138, 99));
    i.patch(0., 0., 0.48, 0.44, 5.4 * s, rgb(88, 164, 104));
    for (u, z) in [(-0.38, -0.35), (0.38, -0.35), (-0.38, 0.35), (0.38, 0.35)] {
        let p = i.p(u, z, 14. * s);
        let b = i.p(u, z, 0.);
        draw_line(b.x, b.y, p.x, p.y, 0.65 * s, rgb(152, 177, 160));
        draw_rectangle(
            p.x - 1.6 * s,
            p.y - 0.6 * s,
            3.2 * s,
            1.2 * s,
            rgb(248, 236, 191),
        );
    }
    i.line(0., -0.2, 0., 0.2, 5.5 * s, 0.6 * s, rgb(211, 228, 187));
    i.line(
        -0.22,
        -0.2,
        0.22,
        -0.2,
        5.5 * s,
        0.6 * s,
        rgb(211, 228, 187),
    );
    i.line(-0.22, 0.2, 0.22, 0.2, 5.5 * s, 0.6 * s, rgb(211, 228, 187));
}
fn airport(i: Iso, tick: u32, n: u32) {
    let s = i.w / 12.;
    i.diamond(rgb(152, 172, 146));
    i.patch(-0.05, 0., 0.37, 1., 0., rgb(93, 119, 117));
    for k in 0..4 {
        i.line(
            -0.05,
            -0.4 + k as f32 * 0.24,
            -0.05,
            -0.29 + k as f32 * 0.24,
            0.,
            0.8 * s,
            rgb(225, 225, 187),
        );
    }
    i.block(0.3, -0.15, 0.3, 0.5, 5. * s, rgb(183, 207, 185));
    i.block(0.3, -0.3, 0.18, 0.16, 13. * s, rgb(160, 189, 176));
    i.raised(0.3, -0.3, 0.25, 0.23, 13. * s, 2. * s, rgb(111, 165, 173));
    let t = ((tick + n % 180) % 180) as f32 / 180. - 0.5;
    let p = i.p(-0.05, t, 2. * s);
    draw_line(
        p.x - 3. * s,
        p.y,
        p.x + 3. * s,
        p.y,
        1. * s,
        rgb(238, 238, 211),
    );
    draw_line(
        p.x - 1.7 * s,
        p.y - 2. * s,
        p.x + 1.7 * s,
        p.y + 2. * s,
        1.1 * s,
        rgb(238, 238, 211),
    );
}

fn overlay(t: &Tile, i: Iso, mode: u8) {
    let bad = rgb(219, 116, 89);
    let good = rgb(85, 214, 173);
    let blue = rgb(88, 186, 232);
    let gray = rgb(112, 139, 137);
    let c = match mode {
        1 => {
            if t.powered {
                rgb(241, 204, 92)
            } else {
                gray
            }
        }
        2 => {
            if t.watered {
                blue
            } else {
                gray
            }
        }
        3 => mix(good, bad, t.traffic as f32 / 100.),
        4 => mix(good, rgb(186, 105, 157), t.pollution as f32 / 100.),
        5 => mix(bad, good, t.land_value as f32 / 100.),
        6 => mix(good, bad, t.crime as f32 / 100.),
        7 => match t.kind {
            Kind::Residential => rgb(94, 212, 132),
            Kind::Commercial => rgb(90, 186, 235),
            Kind::Industrial => rgb(240, 202, 91),
            _ => gray,
        },
        8 => match t.kind {
            Kind::Police
            | Kind::Fire
            | Kind::Hospital
            | Kind::School
            | Kind::University
            | Kind::Park => rgb(138, 218, 217),
            _ => {
                if t.connected {
                    good
                } else {
                    gray
                }
            }
        },
        _ => gray,
    };
    if t.terrain != Terrain::Water || t.kind != Kind::Empty {
        i.diamond(alpha(c, 0.43));
        if !matches!(
            t.kind,
            Kind::Empty | Kind::Road | Kind::Avenue | Kind::Pipe | Kind::PowerLine | Kind::Rail
        ) {
            let p = i.p(0., 0., 2. * i.w / 12.);
            draw_circle(p.x, p.y, 1.9 * i.w / 12., c);
        }
    }
}
fn fire(i: Iso, intensity: u8, tick: u32, n: u32) {
    let s = i.w / 12.;
    let height = (7. + intensity.min(100) as f32 / 14.) * s;
    for k in 0..3 {
        let u = (k as f32 - 1.) * 0.15;
        let p = i.p(u, 0.07, 3. * s);
        let flicker = ((tick + n + k * 7) % 19) as f32 / 19.;
        draw_triangle(
            vec2(p.x - 2. * s, p.y),
            vec2(p.x + 2. * s, p.y),
            vec2(
                p.x + (flicker - 0.5) * 2. * s,
                p.y - height * (0.7 + flicker * 0.3),
            ),
            rgb(240, 119, 60),
        );
        draw_triangle(
            vec2(p.x - 1. * s, p.y),
            vec2(p.x + 1. * s, p.y),
            vec2(p.x, p.y - height * 0.5),
            rgb(255, 220, 107),
        );
    }
    smoke(i, 0., 0., height, tick, n, 1.5);
}
