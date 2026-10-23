//! Procedural architectural recipes and read-only world effects.
use super::*;

fn ring(i: Iso, u: f32, v: f32, z: f32, radius: f32, color: Color) {
    let p = i.p(u, v, z);
    draw_ellipse(p.x, p.y, radius * i.w, radius * i.h, 0., color);
}
fn mast(i: Iso, u: f32, v: f32, height: f32, color: Color) {
    let a = i.p(u, v, 0.);
    let b = i.p(u, v, height * i.w / 12.);
    draw_line(a.x, a.y, b.x, b.y, 0.9 * i.w / 12., color);
}
fn cross(i: Iso, z: f32, color: Color) {
    let p = i.p(0., 0., z);
    let s = i.w / 12.;
    draw_rectangle(p.x - s, p.y - 3. * s, 2. * s, 6. * s, color);
    draw_rectangle(p.x - 3. * s, p.y - s, 6. * s, 2. * s, color);
}
fn court(i: Iso, color: Color) {
    let s = i.w / 12.;
    i.patch(0., 0., 0.84, 0.8, 0., rgb(198, 204, 177));
    i.patch(0., 0., 0.65, 0.62, 0.2 * s, color);
    for v in [-0.28, 0., 0.28] {
        i.line(-0.3, v, 0.3, v, 0.4 * s, 0.55 * s, rgb(224, 235, 207));
    }
    for u in [-0.3, 0.3] {
        i.line(u, -0.28, u, 0.28, 0.4 * s, 0.55 * s, rgb(224, 235, 207));
    }
}
fn hall(i: Iso, seed: u32, color: Color, roof: Color, h: f32) {
    let s = i.w / 12.;
    i.diamond(rgb(170, 187, 157));
    i.shadow(0., 0., 0.8, 0.7, h * s);
    i.block(0., -0.04, 0.8, 0.7, h * s, color);
    i.patch(0., -0.04, 0.74, 0.64, h * s + 0.2 * s, roof);
    i.windows(0., -0.04, 0.8, 0.7, h * s, seed, rgb(147, 192, 194));
}
fn colonnade(i: Iso, color: Color, height: f32) {
    let s = i.w / 12.;
    for u in [-0.3, -0.1, 0.1, 0.3] {
        i.block(u, 0.3, 0.07, 0.08, height * s, color);
    }
    i.raised(0., 0.28, 0.88, 0.2, height * s, 1.5 * s, tint(color, 1.06));
}
fn water_tank(i: Iso, u: f32, v: f32, height: f32, color: Color) {
    let s = i.w / 12.;
    i.block(u, v, 0.3, 0.3, height * s, color);
    ring(i, u, v, height * s, 0.25, tint(color, 1.13));
}
fn tent(i: Iso, u: f32, v: f32, color: Color) {
    let s = i.w / 12.;
    let a = i.p(u - 0.2, v - 0.18, 0.);
    let b = i.p(u + 0.2, v - 0.18, 0.);
    let c = i.p(u + 0.2, v + 0.18, 0.);
    let d = i.p(u - 0.2, v + 0.18, 0.);
    let p = i.p(u - 0.2, v, 5. * s);
    let q = i.p(u + 0.2, v, 5. * s);
    quad(a, b, q, p, tint(color, 1.08));
    quad(p, q, c, d, color);
    draw_triangle(b, c, q, tint(color, 0.68));
}

/// Each catalog entry has an identifying silhouette rather than a generic service box.
pub(super) fn facility(city: &City, t: &Tile, i: Iso, x: usize, y: usize, n: u32, v: View) {
    let s = i.w / 12.;
    match t.kind {
        Kind::Gas => {
            hall(i, n, rgb(174, 185, 173), rgb(112, 151, 153), 6.);
            water_tank(i, -0.23, -0.22, 10., rgb(174, 203, 195));
            i.raised(0.23, -0.24, 0.13, 0.13, 6. * s, 13. * s, rgb(147, 162, 151));
            i.line(-0.35, 0.35, 0.35, 0.35, 2. * s, 1.4 * s, rgb(218, 193, 128));
        }
        Kind::Solar => {
            i.diamond(rgb(140, 173, 127));
            for u in [-0.25, 0.22] {
                for z in [-0.23, 0.23] {
                    i.raised(u, z, 0.4, 0.34, s, 1.4 * s, rgb(57, 113, 143));
                    i.line(
                        u - 0.19,
                        z,
                        u + 0.19,
                        z,
                        2.6 * s,
                        0.5 * s,
                        rgb(128, 196, 211),
                    );
                    i.line(
                        u,
                        z - 0.16,
                        u,
                        z + 0.16,
                        2.6 * s,
                        0.5 * s,
                        rgb(128, 196, 211),
                    );
                }
            }
        }
        Kind::Hydro => {
            i.diamond(rgb(48, 137, 154));
            i.block(-0.07, 0., 0.4, 0.95, 10. * s, rgb(187, 204, 188));
            for z in [-0.3, 0., 0.3] {
                i.line(0.15, z, 0.4, z, 0., 2. * s, rgb(157, 226, 216));
            }
            i.raised(
                -0.07,
                -0.15,
                0.42,
                0.38,
                10. * s,
                3. * s,
                rgb(112, 155, 157),
            );
        }
        Kind::Fusion => {
            hall(i, n, rgb(164, 197, 195), rgb(79, 150, 168), 7.);
            let p = i.p(0., 0., 8. * s);
            draw_circle(p.x, p.y - 2. * s, 4.5 * s, rgb(127, 213, 216));
            draw_circle_lines(p.x, p.y - 2. * s, 5.6 * s, 1.1 * s, rgb(224, 232, 207));
            i.raised(-0.3, -0.2, 0.12, 0.12, 7. * s, 8. * s, rgb(120, 171, 179));
        }
        Kind::WaterTower => {
            i.diamond(rgb(143, 181, 146));
            for [u, z] in [[-0.2, -0.2], [0.2, -0.2], [-0.2, 0.2], [0.2, 0.2]] {
                mast(i, u, z, 13., rgb(151, 179, 175));
            }
            i.raised(0., 0., 0.55, 0.55, 11. * s, 5. * s, rgb(167, 211, 211));
            ring(i, 0., 0., 16. * s, 0.38, rgb(210, 231, 220));
        }
        Kind::SewageOutlet => {
            i.diamond(rgb(133, 169, 151));
            i.block(-0.22, -0.15, 0.4, 0.5, 5. * s, rgb(179, 196, 173));
            for z in [-0.18, 0.18] {
                i.line(-0.05, z, 0.48, z, 2. * s, 2.2 * s, rgb(132, 161, 157));
                i.line(0.2, z, 0.5, z, 0., 1.8 * s, rgb(69, 151, 155));
            }
        }
        Kind::Incinerator => {
            hall(i, n, rgb(189, 145, 118), rgb(106, 139, 140), 9.);
            i.raised(
                -0.25,
                -0.25,
                0.14,
                0.14,
                9. * s,
                15. * s,
                rgb(164, 133, 118),
            );
            smoke(city, i, -0.25, -0.25, 25. * s, v.tick, n, 0.65);
            let p = i.p(0.1, 0.37, 3. * s);
            draw_circle(p.x, p.y, 1.6 * s, rgb(239, 160, 83));
        }
        Kind::Recycling => {
            hall(i, n, rgb(138, 185, 154), rgb(93, 153, 146), 6.);
            for (u, c) in [
                (-0.24, rgb(97, 171, 184)),
                (0., rgb(226, 190, 104)),
                (0.24, rgb(92, 164, 116)),
            ] {
                i.block(u, 0.28, 0.16, 0.24, 3. * s, c);
            }
            let p = i.p(0., 0., 7. * s);
            draw_poly_lines(p.x, p.y, 3, 2.8 * s, -90., 0.9 * s, rgb(222, 239, 208));
        }
        Kind::Prison => {
            hall(i, n, rgb(161, 171, 165), rgb(101, 130, 131), 9.);
            for [u, z] in [[-0.4, -0.4], [0.4, -0.4], [-0.4, 0.4], [0.4, 0.4]] {
                i.block(u, z, 0.15, 0.15, 13. * s, rgb(178, 187, 174));
            }
            for u in [-0.44, 0.44] {
                i.block(u, 0., 0.045, 0.86, 4. * s, rgb(132, 151, 139));
            }
        }
        Kind::FireHQ => {
            service(i, n, rgb(190, 107, 85), rgb(220, 164, 111), 1);
            i.raised(0.25, -0.2, 0.23, 0.26, 8. * s, 9. * s, rgb(192, 136, 105));
            cross(i, 18. * s, rgb(248, 213, 145));
        }
        Kind::Clinic => {
            service(i, n, rgb(222, 226, 207), rgb(111, 173, 174), 3);
            cross(i, 9.5 * s, rgb(219, 99, 88));
        }
        Kind::Cemetery => {
            i.diamond(rgb(114, 160, 119));
            i.patch(0., 0., 0.14, 1., 0., rgb(195, 193, 157));
            for z in [-0.27, 0.03, 0.3] {
                for u in [-0.25, 0.23] {
                    i.block(u, z, 0.12, 0.075, 2.8 * s, rgb(186, 197, 181));
                }
            }
            lot_conifer(city, [x, y], i, [-0.3, -0.3], n);
        }
        Kind::Library | Kind::HeritageMuseum => {
            let c = if t.kind == Kind::Library {
                rgb(209, 199, 165)
            } else {
                rgb(208, 183, 154)
            };
            hall(i, n, c, rgb(108, 155, 153), 9.);
            colonnade(i, rgb(228, 218, 188), 6.);
            if t.kind == Kind::HeritageMuseum {
                let a = i.p(-0.43, 0.3, 10. * s);
                let b = i.p(0.43, 0.3, 10. * s);
                let top = i.p(0., 0.3, 14. * s);
                draw_triangle(a, b, top, rgb(198, 177, 147));
            } else {
                i.raised(0., 0., 0.22, 0.42, 9. * s, 2. * s, rgb(231, 220, 176));
            }
        }
        Kind::BusStop => {
            i.diamond(rgb(182, 194, 176));
            i.block(0., 0.05, 0.48, 0.19, 2. * s, rgb(141, 161, 153));
            for u in [-0.3, 0.3] {
                mast(i, u, -0.1, 7., rgb(134, 165, 162));
            }
            i.patch(0., -0.1, 0.72, 0.42, 7. * s, rgb(104, 172, 184));
            mast(i, 0.35, 0.25, 9., rgb(180, 197, 181));
            let p = i.p(0.35, 0.25, 9. * s);
            draw_rectangle(p.x - s, p.y - s, 2. * s, 2. * s, line_color(city, x, y));
        }
        Kind::SubwayStation => {
            i.diamond(rgb(181, 192, 177));
            i.patch(0., 0., 0.62, 0.55, 0., rgb(68, 105, 116));
            for k in 0..4 {
                i.line(
                    -0.28,
                    -0.18 + k as f32 * 0.1,
                    0.28,
                    -0.18 + k as f32 * 0.1,
                    0.,
                    0.7 * s,
                    rgb(153, 183, 180),
                );
            }
            for u in [-0.32, 0.32] {
                mast(i, u, 0., 8., rgb(152, 179, 176));
            }
            i.patch(0., 0., 0.77, 0.2, 8. * s, line_color(city, x, y));
        }
        Kind::FreightStation => {
            hall(i, n, rgb(178, 181, 154), rgb(95, 150, 155), 7.);
            for (u, c) in [
                (-0.25, rgb(202, 151, 103)),
                (0., rgb(133, 177, 174)),
                (0.25, rgb(217, 192, 129)),
            ] {
                i.block(u, 0.24, 0.2, 0.32, 3. * s, c);
            }
            i.line(-0.45, -0.4, 0.45, -0.4, 0., s, rgb(96, 122, 120));
        }
        Kind::Airfield => {
            airport(i, v.tick, n, !city.ext.enabled);
            let p = i.p(-0.3, 0.25, 7. * s);
            mast(i, -0.3, 0.25, 7., rgb(181, 199, 179));
            draw_triangle(
                p,
                p + vec2(4. * s, s),
                p + vec2(0., 2. * s),
                rgb(232, 165, 97),
            );
        }
        Kind::Marina => {
            i.diamond(rgb(63, 144, 157));
            i.patch(0., 0., 0.16, 1., 0., rgb(186, 174, 134));
            for z in [-0.3, 0., 0.3] {
                i.patch(0.15, z, 0.72, 0.08, 0., rgb(189, 176, 139));
                let p = i.p(-0.23, z, 0.);
                draw_ellipse(p.x, p.y, 2.1 * s, s, 0., rgb(224, 231, 214));
                draw_triangle(
                    p,
                    p + vec2(0., -5. * s),
                    p + vec2(2.8 * s, -s),
                    rgb(226, 211, 162),
                );
            }
        }
        Kind::MaintenanceDepot => {
            hall(i, n, rgb(177, 188, 157), rgb(113, 153, 149), 6.);
            i.block(0.06, 0.3, 0.5, 0.24, 3. * s, rgb(226, 184, 92));
            i.patch(0.18, 0.3, 0.13, 0.21, 3.2 * s, rgb(97, 145, 151));
            cross(i, 7.5 * s, rgb(224, 204, 145));
        }
        Kind::Farm => {
            i.diamond(rgb(152, 165, 102));
            for k in 0..5 {
                i.patch(
                    -0.36 + k as f32 * 0.18,
                    0.,
                    0.075,
                    0.9,
                    0.,
                    if k % 2 == 0 {
                        rgb(103, 147, 90)
                    } else {
                        rgb(187, 173, 113)
                    },
                );
            }
            i.block(-0.15, -0.2, 0.36, 0.36, 4.5 * s, rgb(185, 123, 99));
            tent(
                Iso {
                    y: i.y - 4.5 * s,
                    ..i
                },
                -0.15,
                -0.2,
                rgb(148, 107, 87),
            );
            i.block(0.28, -0.25, 0.15, 0.17, 7. * s, rgb(208, 206, 172));
        }
        Kind::Hotel => {
            hall(i, n, rgb(218, 219, 200), rgb(112, 170, 177), 23.);
            for k in 0..4 {
                i.line(
                    -0.4,
                    0.32,
                    0.4,
                    0.32,
                    (4. + k as f32 * 4.) * s,
                    0.8 * s,
                    rgb(174, 194, 182),
                );
            }
            i.patch(0.2, 0.32, 0.25, 0.25, 0., rgb(98, 196, 207));
        }
        Kind::Playground => {
            i.diamond(rgb(125, 175, 119));
            i.patch(0., 0., 0.75, 0.65, 0., rgb(213, 193, 147));
            for u in [-0.25, 0.23] {
                mast(i, u, -0.1, 7., rgb(208, 128, 94));
            }
            i.line(-0.25, -0.1, 0.23, -0.1, 7. * s, 1.2 * s, rgb(225, 173, 105));
            let a = i.p(0.2, 0.1, 5. * s);
            let b = i.p(0.2, 0.4, 0.);
            draw_line(a.x, a.y, b.x, b.y, 2. * s, rgb(97, 170, 191));
            lot_tree(city, [x, y], i, [-0.33, 0.29], n, 0.55);
        }
        Kind::Plaza => {
            i.diamond(rgb(191, 199, 178));
            for k in 0..3 {
                i.line(
                    -0.45,
                    -0.3 + k as f32 * 0.3,
                    0.45,
                    -0.3 + k as f32 * 0.3,
                    0.,
                    0.5 * s,
                    rgb(165, 183, 167),
                );
            }
            ring(i, 0., 0., s, 0.25, rgb(147, 180, 163));
            i.block(0., 0., 0.16, 0.16, 5. * s, rgb(220, 207, 166));
        }
        Kind::BotanicalGarden => {
            i.diamond(rgb(101, 170, 117));
            i.block(0., 0., 0.62, 0.62, 5. * s, rgb(142, 196, 181));
            let p = i.p(0., 0., 6. * s);
            draw_ellipse(p.x, p.y - 1.5 * s, 4.8 * s, 3.3 * s, 0., rgb(177, 225, 212));
            for z in [-0.32, 0.32] {
                for u in [-0.3, 0., 0.3] {
                    let p = i.p(u, z, 0.);
                    draw_circle(
                        p.x,
                        p.y,
                        1.2 * s,
                        if u < 0. {
                            rgb(229, 173, 151)
                        } else {
                            rgb(230, 210, 139)
                        },
                    );
                }
            }
        }
        Kind::DogPark => {
            i.diamond(rgb(119, 170, 113));
            for u in [-0.4, 0.4] {
                i.line(u, -0.4, u, 0.4, 2. * s, 0.7 * s, rgb(155, 174, 145));
            }
            ring(i, 0.2, 0.12, 0., 0.15, rgb(208, 195, 146));
            i.block(-0.1, 0.06, 0.22, 0.1, 1.8 * s, rgb(150, 119, 86));
            i.block(-0.22, 0.06, 0.09, 0.1, 3. * s, rgb(160, 128, 91));
        }
        Kind::SkatePark => {
            i.diamond(rgb(161, 180, 159));
            ring(i, 0., 0., 0., 0.39, rgb(194, 207, 190));
            ring(i, 0., 0., 0.2 * s, 0.29, rgb(126, 156, 155));
            i.line(-0.3, 0.26, 0.25, 0.26, 3. * s, 0.8 * s, rgb(109, 140, 141));
        }
        Kind::WaterfrontPark => {
            i.diamond(rgb(124, 177, 142));
            i.patch(0.24, 0., 0.25, 1., 0., rgb(197, 186, 145));
            i.line(0.35, -0.4, 0.35, 0.4, 3. * s, 0.6 * s, rgb(203, 216, 191));
            lot_tree(city, [x, y], i, [-0.2, -0.2], n, 0.85);
            lot_tree(city, [x, y], i, [-0.2, 0.2], n + 1, 0.7);
        }
        Kind::NatureReserve => {
            i.diamond(rgb(93, 151, 109));
            lot_conifer(city, [x, y], i, [-0.23, -0.2], n);
            lot_tree(city, [x, y], i, [0.2, 0.1], n + 1, 1.05);
            i.patch(-0.1, 0.32, 0.7, 0.1, 0., rgb(183, 181, 139));
        }
        Kind::CityHall => {
            hall(i, n, rgb(219, 210, 177), rgb(105, 153, 153), 10.);
            colonnade(i, rgb(232, 222, 191), 7.);
            i.raised(0., -0.1, 0.28, 0.28, 10. * s, 5. * s, rgb(209, 208, 179));
            ring(i, 0., -0.1, 15. * s, 0.22, rgb(139, 186, 178));
        }
        Kind::Market => {
            i.diamond(rgb(181, 190, 156));
            for (u, c) in [(-0.23, rgb(214, 147, 114)), (0.23, rgb(110, 177, 178))] {
                i.block(u, 0., 0.35, 0.66, 3. * s, rgb(199, 185, 146));
                i.patch(u, 0., 0.4, 0.75, 4. * s, c);
                i.line(
                    u - 0.16,
                    0.,
                    u + 0.16,
                    0.,
                    4.2 * s,
                    0.7 * s,
                    rgb(235, 224, 179),
                );
            }
        }
        Kind::Fountain => {
            i.diamond(rgb(189, 202, 176));
            ring(i, 0., 0., 0., 0.43, rgb(216, 216, 187));
            ring(i, 0., 0., 0.3 * s, 0.35, rgb(99, 190, 197));
            let p = i.p(0., 0., 0.);
            draw_line(p.x, p.y, p.x, p.y - 6. * s, 1.1 * s, rgb(184, 232, 219));
            draw_ellipse(p.x, p.y - 4. * s, 2.2 * s, 1.1 * s, 0., rgb(165, 222, 212));
        }
        Kind::Monument => {
            i.diamond(rgb(174, 190, 163));
            i.block(0., 0., 0.45, 0.45, 2. * s, rgb(207, 204, 174));
            i.raised(0., 0., 0.17, 0.17, 2. * s, 15. * s, rgb(218, 216, 185));
            let p = i.p(0., 0., 19. * s);
            let a = i.p(-0.085, -0.085, 17. * s);
            let b = i.p(0.085, 0.085, 17. * s);
            draw_triangle(a, b, p, rgb(238, 229, 195));
        }
        Kind::ClockTower => {
            i.diamond(rgb(178, 190, 159));
            i.block(0., 0., 0.36, 0.36, 23. * s, rgb(198, 182, 151));
            let p = i.p(0., 0.19, 19. * s);
            draw_circle(p.x, p.y, 2.6 * s, rgb(238, 230, 190));
            draw_line(p.x, p.y, p.x, p.y - 1.5 * s, 0.6 * s, rgb(94, 125, 130));
            draw_line(p.x, p.y, p.x + 1.3 * s, p.y, 0.6 * s, rgb(94, 125, 130));
            i.raised(0., 0., 0.42, 0.42, 23. * s, 2. * s, rgb(112, 154, 152));
        }
        Kind::ConcertHall => {
            hall(i, n, rgb(202, 206, 184), rgb(109, 169, 176), 6.);
            for u in [-0.25, 0., 0.25] {
                let p = i.p(u, -0.04, 8. * s);
                draw_triangle(
                    p + vec2(-3. * s, 1. * s),
                    p + vec2(4. * s, 2. * s),
                    p + vec2(0., -6. * s),
                    rgb(233, 230, 203),
                );
            }
        }
        Kind::Observatory => {
            hall(i, n, rgb(201, 205, 182), rgb(123, 172, 174), 7.);
            let p = i.p(0., 0., 8. * s);
            draw_circle(p.x, p.y - 2. * s, 4.5 * s, rgb(202, 224, 211));
            draw_line(
                p.x,
                p.y - 3. * s,
                p.x + 5. * s,
                p.y - 7. * s,
                2.1 * s,
                rgb(100, 145, 156),
            );
        }
        Kind::ScienceCenter => {
            hall(i, n, rgb(180, 205, 193), rgb(86, 156, 174), 8.);
            let p = i.p(0., 0., 9. * s);
            draw_circle(p.x, p.y - 2. * s, 4.5 * s, rgb(127, 202, 208));
            for angle in [0., 60., 120.] {
                draw_ellipse_lines(
                    p.x,
                    p.y - 2. * s,
                    5.3 * s,
                    2.3 * s,
                    angle,
                    0.7 * s,
                    rgb(220, 231, 201),
                );
            }
        }
        Kind::Aquarium => {
            hall(i, n, rgb(174, 203, 195), rgb(74, 154, 178), 6.);
            water_tank(i, 0., 0., 11., rgb(114, 188, 202));
            ring(i, 0., 0., 11. * s, 0.3, rgb(178, 222, 218));
        }
        Kind::Zoo => {
            i.diamond(rgb(114, 169, 119));
            i.patch(-0.2, 0., 0.4, 0.75, 0., rgb(196, 185, 140));
            ring(i, 0.23, 0.1, 0., 0.22, rgb(99, 180, 184));
            i.block(-0.2, -0.1, 0.2, 0.16, 3. * s, rgb(205, 174, 107));
            i.block(-0.28, -0.15, 0.07, 0.08, 7. * s, rgb(207, 178, 111));
            lot_tree(city, [x, y], i, [0.22, -0.28], n, 0.7);
        }
        Kind::SportsCenter => {
            court(i, rgb(103, 166, 150));
            i.block(-0.28, -0.28, 0.3, 0.28, 5. * s, rgb(199, 209, 181));
            i.patch(-0.28, -0.28, 0.33, 0.31, 5.2 * s, rgb(104, 156, 161));
        }
        Kind::Beach => {
            i.diamond(rgb(218, 207, 156));
            let p = i.p(0., 0., 0.);
            draw_line(p.x, p.y, p.x, p.y - 5. * s, 0.7 * s, rgb(166, 156, 122));
            draw_triangle(
                p + vec2(-4. * s, -5. * s),
                p + vec2(4. * s, -5. * s),
                p + vec2(0., -8. * s),
                rgb(219, 143, 111),
            );
            i.patch(0.25, 0.22, 0.16, 0.35, 0., rgb(137, 191, 190));
        }
        Kind::Campground => {
            i.diamond(rgb(111, 162, 112));
            tent(i, -0.2, -0.2, rgb(215, 170, 100));
            tent(i, 0.22, 0.17, rgb(113, 170, 179));
            ring(i, -0.2, 0.25, 0., 0.1, rgb(161, 151, 110));
            lot_conifer(city, [x, y], i, [0.25, -0.3], n);
        }
        Kind::Fairground => {
            i.diamond(rgb(177, 187, 144));
            let p = i.p(0., 0., 8. * s);
            let phase = if v.calm {
                0.
            } else {
                (v.tick % 720) as f32 * 0.006
            };
            draw_circle_lines(p.x, p.y, 6. * s, 0.8 * s, rgb(219, 198, 143));
            for k in 0..8 {
                let a = phase + k as f32 * std::f32::consts::TAU / 8.;
                let q = p + vec2(a.cos(), a.sin()) * 6. * s;
                draw_line(p.x, p.y, q.x, q.y, 0.4 * s, rgb(150, 177, 164));
                draw_rectangle(
                    q.x - s,
                    q.y - s,
                    2. * s,
                    2. * s,
                    if k % 2 == 0 {
                        rgb(219, 146, 113)
                    } else {
                        rgb(120, 183, 190)
                    },
                );
            }
            mast(i, -0.2, 0., 8., rgb(188, 185, 149));
            mast(i, 0.2, 0., 8., rgb(188, 185, 149));
        }
        Kind::Arcology => {
            i.diamond(rgb(132, 177, 136));
            for k in 0..3 {
                let w = 0.88 - k as f32 * 0.18;
                let h = (k + 1) as f32 * 12. * s;
                i.raised(
                    0.,
                    0.,
                    w,
                    w,
                    k as f32 * 12. * s,
                    12. * s,
                    rgb(164, 204, 193),
                );
                Iso {
                    y: i.y - k as f32 * 12. * s,
                    ..i
                }
                .windows(0., 0., w, w, 12. * s, n, rgb(194, 227, 211));
                i.patch(0., 0., w - 0.05, w - 0.05, h + 0.2 * s, rgb(106, 165, 127));
            }
            mast(i, 0., 0., 43., rgb(184, 214, 196));
        }
        Kind::Spaceport => {
            i.diamond(rgb(157, 177, 155));
            ring(i, 0., 0., 0., 0.44, rgb(182, 197, 180));
            i.block(0., 0., 0.17, 0.18, 20. * s, rgb(223, 231, 211));
            let p = i.p(0., 0., 24. * s);
            draw_triangle(
                i.p(-0.09, 0., 20. * s),
                i.p(0.09, 0., 20. * s),
                p,
                rgb(172, 202, 193),
            );
            i.raised(-0.27, -0.22, 0.12, 0.14, 0., 24. * s, rgb(133, 160, 151));
            i.line(-0.27, -0.22, 0., 0., 17. * s, 0.7 * s, rgb(167, 190, 169));
        }
        Kind::WorldExpo => {
            i.diamond(rgb(158, 187, 149));
            let p = i.p(0., 0., 5. * s);
            draw_circle(p.x, p.y - 2. * s, 7. * s, rgb(155, 207, 201));
            for angle in [0., 45., 90., 135.] {
                draw_ellipse_lines(
                    p.x,
                    p.y - 2. * s,
                    7. * s,
                    3.6 * s,
                    angle,
                    0.65 * s,
                    rgb(218, 231, 204),
                );
            }
            i.patch(0., 0.35, 0.3, 0.2, 0., rgb(208, 206, 167));
        }
        _ => {}
    }
}

fn line_color(city: &City, x: usize, y: usize) -> Color {
    let tile = (y * usize::from(city.width) + x) as u32;
    city.ext
        .transport
        .lines
        .iter()
        .find(|line| line.enabled && line.stops.contains(&tile))
        .map(|line| rgb(line.color[0], line.color[1], line.color[2]))
        .unwrap_or(rgb(87, 158, 181))
}

pub(super) fn grass(city: &City, base: Color) -> Color {
    use crate::world::Climate;
    let season = city.ext.day % 360;
    if city.ext.weather.kind == crate::world::WeatherKind::Snow {
        return mix(base, rgb(215, 227, 208), 0.74);
    }
    match city.ext.climate {
        Climate::Tropical => mix(base, rgb(91, 161, 119), 0.5),
        Climate::Alpine if !(60..300).contains(&season) => mix(base, rgb(208, 221, 202), 0.7),
        Climate::Alpine => mix(base, rgb(124, 159, 135), 0.42),
        Climate::Temperate if (240..300).contains(&season) => mix(base, rgb(171, 167, 110), 0.35),
        _ => base,
    }
}
pub(super) fn water_color(city: &City, index: usize, base: Color) -> Color {
    let depth = city
        .ext
        .visual_tile(index)
        .map(|c| c.water_depth)
        .unwrap_or(0);
    let base = tint(base, 1.04 - f32::from(depth.min(30)) / 300.);
    if city.ext.enabled && city.ext.weather.temperature < -4 {
        mix(base, rgb(169, 207, 213), 0.78)
    } else {
        base
    }
}
pub(super) fn plant(
    city: &City,
    at: [usize; 2],
    i: Iso,
    position: [f32; 2],
    n: u32,
    size: f32,
    evergreen: bool,
) {
    use crate::world::Climate;
    let index = at[1] * usize::from(city.width) + at[0];
    let age = city
        .ext
        .visual_tile(index)
        .map(|c| c.tree_age)
        .unwrap_or(600);
    let species = city
        .ext
        .visual_tile(index)
        .map(|c| c.tree_species)
        .unwrap_or(0);
    let n = n.wrapping_add(u32::from(species) * 971);
    let maturity = if city.ext.enabled {
        0.48 + f32::from(age.min(1000)) / 1700.
    } else {
        1.
    };
    let size = size * maturity;
    let p = i.p(position[0], position[1], 0.);
    let s = i.w / 12. * size;
    if city.ext.climate == Climate::Tropical && !evergreen && species.is_multiple_of(2) {
        let top = p + vec2(
            (1.1 + (f32::from(city.ext.weather.wind_x) - f32::from(city.ext.weather.wind_y))
                * 0.025)
                * s,
            -12. * s,
        );
        draw_line(
            p.x,
            p.y,
            p.x + 0.3 * s,
            p.y - 6. * s,
            1.2 * s,
            rgb(132, 119, 80),
        );
        draw_line(
            p.x + 0.3 * s,
            p.y - 6. * s,
            top.x,
            top.y,
            1.1 * s,
            rgb(151, 137, 91),
        );
        for k in 0..6 {
            let a = k as f32 * std::f32::consts::TAU / 6.;
            let tip = top + vec2(a.cos() * 5. * s, a.sin() * 2.2 * s + 1.2 * s);
            draw_triangle(
                top,
                tip,
                top + vec2((a + 0.4).cos() * 3.2 * s, (a + 0.4).sin() * 2. * s),
                if k % 2 == 0 {
                    rgb(54, 135, 87)
                } else {
                    rgb(73, 154, 102)
                },
            );
        }
    } else if evergreen || city.ext.climate == Climate::Alpine {
        let local = Iso {
            w: i.w * size,
            h: i.h * size,
            x: p.x,
            y: p.y,
        };
        conifer(local, 0., 0., n);
    } else {
        tree(i, position[0], position[1], n, size);
        if (240..300).contains(&(city.ext.day % 360)) {
            draw_poly(
                p.x - s,
                p.y - 8. * s,
                7,
                2.3 * s,
                0.,
                alpha(rgb(189, 151, 91), 0.65),
            );
        }
    }
}

pub(super) fn terrain_detail(city: &City, index: usize, i: Iso, v: View) {
    let Some(cell) = city.ext.visual_tile(index) else {
        return;
    };
    let x = index % usize::from(city.width);
    let y = index / usize::from(city.width);
    let s = i.w / 12.;
    for (dx, dy) in [(1, 0), (0, 1)] {
        let neighbor = city
            .idx(x as i32 + dx, y as i32 + dy)
            .and_then(|n| city.ext.visual_tile(n));
        let other = neighbor.map(|c| c.elevation).unwrap_or(cell.elevation);
        let drop = f32::from(cell.elevation - other) * i.h / 16.;
        if drop > 0.3 {
            let (a, b) = if dx == 1 {
                (i.p(0.5, -0.5, 0.), i.p(0.5, 0.5, 0.))
            } else {
                (i.p(-0.5, 0.5, 0.), i.p(0.5, 0.5, 0.))
            };
            quad(
                a,
                b,
                b + vec2(0., drop),
                a + vec2(0., drop),
                if dx == 1 {
                    rgb(102, 124, 97)
                } else {
                    rgb(130, 148, 111)
                },
            );
        }
    }
    if city.tiles[index].kind == Kind::Empty && city.tiles[index].terrain != Terrain::Water {
        if cell.ore > 300 {
            for k in 0..3 {
                let p = i.p(-0.18 + k as f32 * 0.17, -0.12, 0.);
                draw_poly(
                    p.x,
                    p.y - s,
                    5,
                    (1.2 + k as f32 * 0.3) * s,
                    15.,
                    if k % 2 == 0 {
                        rgb(136, 155, 147)
                    } else {
                        rgb(174, 170, 137)
                    },
                );
            }
        }
        if cell.oil > 650 {
            ring(i, 0.17, 0.18, 0., 0.12, alpha(rgb(51, 92, 83), 0.6));
        }
    }
    if cell.water_depth > 0
        && city.tiles[index].terrain == Terrain::Water
        && !v.calm
        && city.ext.weather.temperature >= -4
    {
        let phase = (v.tick.wrapping_add(index as u32) % 180) as f32 / 180.;
        let p = i.p(phase * 0.4 - 0.2, -0.12, 0.);
        draw_line(
            p.x - 2.5 * s,
            p.y,
            p.x + 2.5 * s,
            p.y,
            0.45 * s,
            alpha(rgb(180, 229, 216), 0.15),
        );
    }
}

pub(super) fn lot_stage(city: &City, index: usize, t: &Tile, i: Iso, n: u32, v: View) -> bool {
    let Some(cell) = city.ext.visual_tile(index) else {
        return false;
    };
    let s = i.w / 12.;
    if cell.rubble {
        i.diamond(rgb(153, 151, 128));
        for k in 0..7 {
            let u = ((n >> (k * 3)) % 7) as f32 / 10. - 0.3;
            let z = ((n >> (k * 2 + 1)) % 7) as f32 / 10. - 0.3;
            i.block(
                u,
                z,
                0.12,
                0.12,
                (1. + (k % 3) as f32) * s,
                if k % 2 == 0 {
                    rgb(148, 154, 140)
                } else {
                    rgb(175, 162, 134)
                },
            );
        }
        return true;
    }
    if cell.construction < 100
        && !(cell.construction == 0 && t.kind.is_zone() && t.level == 0)
        && !street(t.kind)
        && !matches!(
            t.kind,
            Kind::Empty | Kind::Rail | Kind::PowerLine | Kind::Pipe | Kind::Subway
        )
    {
        i.diamond(rgb(174, 166, 127));
        i.block(0., 0., 0.74, 0.7, 1.5 * s, rgb(176, 180, 153));
        let h = (4. + f32::from(cell.construction) * 0.11) * s;
        if cell.construction >= 25 {
            for [u, z] in [[-0.32, -0.3], [0.32, -0.3], [-0.32, 0.3], [0.32, 0.3]] {
                mast(i, u, z, h / s, rgb(165, 146, 106));
            }
            for k in 0..3 {
                let h = h * (k + 1) as f32 / 3.;
                for z in [-0.3, 0.3] {
                    i.line(-0.32, z, 0.32, z, h, 0.6 * s, rgb(204, 189, 139));
                }
            }
        }
        let crane = i.p(-0.32, -0.28, h + 8. * s);
        mast(i, -0.32, -0.28, h / s + 8., rgb(222, 184, 96));
        draw_line(
            crane.x - 2. * s,
            crane.y,
            crane.x + 11. * s,
            crane.y,
            0.9 * s,
            rgb(225, 189, 104),
        );
        let swing = if v.calm {
            0.
        } else {
            ((v.tick % 360) as f32 * 0.015).sin() * s
        };
        draw_line(
            crane.x + 8. * s + swing,
            crane.y,
            crane.x + 8. * s + swing,
            crane.y + 7. * s,
            0.5 * s,
            rgb(117, 140, 130),
        );
        return true;
    }
    false
}
pub(super) fn wear(city: &City, index: usize, t: &Tile, i: Iso, n: u32, era: u8) {
    let Some(c) = city.ext.visual_tile(index) else {
        return;
    };
    let s = i.w / 12.;
    if era > 0 && t.kind.is_zone() && t.level > 0 {
        let height = building_height(t, n);
        i.patch(
            -0.1,
            -0.1,
            0.2,
            0.28,
            height * s + 0.3 * s,
            if era > 2 {
                rgb(83, 151, 169)
            } else {
                rgb(164, 193, 174)
            },
        );
    }
    if c.abandoned && t.kind.is_zone() && t.level > 0 {
        let wall = match t.kind {
            Kind::Residential if t.level == 1 => 6.,
            Kind::Commercial if t.level == 1 => 6.,
            Kind::Industrial => 5.5 + f32::from(t.level) * 1.5,
            _ => building_height(t, n),
        };
        i.windows(0., 0., 0.67, 0.67, wall * s, n, rgb(96, 118, 110));
    }
    if c.abandoned && t.kind != Kind::Empty {
        i.diamond(alpha(rgb(143, 124, 91), 0.25));
        i.line(-0.25, 0.25, 0.25, 0.25, 3. * s, 1.8 * s, rgb(143, 125, 94));
        let p = i.p(0.2, 0.25, 0.);
        draw_line(
            p.x,
            p.y,
            p.x + 2. * s,
            p.y - 3. * s,
            0.7 * s,
            rgb(101, 144, 92),
        );
    }
    if c.condition < 60 && t.kind.is_zone() && t.level > 0 {
        let h = building_height(t, n).min(12.) * 0.5 * s;
        i.line(-0.25, 0.355, -0.07, 0.355, h, 0.55 * s, rgb(116, 119, 98));
        i.line(
            -0.07,
            0.355,
            0.02,
            0.355,
            h - 1.2 * s,
            0.55 * s,
            rgb(116, 119, 98),
        );
    }
    if c.condition < 60 && t.kind != Kind::Empty {
        for k in 0..3 {
            let u = -0.22 + k as f32 * 0.2;
            let z = ((n >> (k * 3)) % 6) as f32 / 10. - 0.25;
            i.line(
                u,
                z,
                u + 0.12,
                z + 0.08,
                0.,
                0.55 * s,
                alpha(rgb(61, 87, 78), 0.5),
            );
        }
    }
    if c.flood > 0 {
        i.diamond(alpha(
            rgb(70, 156, 178),
            (f32::from(c.flood) / 150.).min(0.65),
        ));
        i.line(
            -0.35,
            0.1,
            0.35,
            0.1,
            0.,
            0.6 * s,
            alpha(rgb(177, 228, 214), 0.5),
        );
    }
}

pub(super) fn road_detail(city: &City, t: &Tile, i: Iso, x: usize, y: usize, v: View) {
    let s = i.w / 12.;
    let index = y * usize::from(city.width) + x;
    let orientation = city
        .ext
        .visual_tile(index)
        .map(|c| c.orientation)
        .unwrap_or(0);
    match t.kind {
        Kind::OneWay => {
            let along = orientation.is_multiple_of(2);
            let p = i.p(0., 0., 0.);
            let direction = if orientation < 2 { 1. } else { -1. };
            let axis = if along {
                vec2(i.w * 0.16, i.h * 0.16)
            } else {
                vec2(-i.w * 0.16, i.h * 0.16)
            } * direction;
            let tip = p + axis;
            let wing = vec2(-axis.y, axis.x).normalize() * 1.1 * s;
            draw_line(
                p.x - axis.x,
                p.y - axis.y,
                tip.x,
                tip.y,
                0.7 * s,
                rgb(228, 230, 196),
            );
            draw_triangle(
                tip,
                tip - axis * 0.8 + wing,
                tip - axis * 0.8 - wing,
                rgb(228, 230, 196),
            );
        }
        Kind::Highway => {
            i.line(-0.5, -0.06, 0.5, -0.06, 0., 0.6 * s, rgb(226, 197, 126));
            i.line(-0.5, 0.06, 0.5, 0.06, 0., 0.6 * s, rgb(226, 197, 126));
            i.line(-0.5, 0.3, 0.5, 0.3, 0., 0.5 * s, rgb(206, 216, 201));
        }
        Kind::Roundabout => {
            ring(i, 0., 0., 0., 0.22, rgb(125, 166, 117));
            ring(i, 0., 0., 0.1 * s, 0.12, rgb(183, 199, 156));
        }
        Kind::Ramp => {
            i.line(-0.4, -0.2, 0.4, 0.2, 0., 0.6 * s, rgb(225, 220, 171));
        }
        Kind::TunnelRoad => {
            i.raised(0., -0.18, 0.82, 0.3, 0., 6. * s, rgb(131, 157, 139));
            i.raised(0., -0.18, 0.52, 0.32, 0., 4.3 * s, rgb(57, 84, 88));
            i.line(
                -0.39,
                -0.32,
                0.39,
                -0.32,
                6.2 * s,
                0.8 * s,
                rgb(204, 210, 176),
            );
        }
        _ => {}
    }
    if t.terrain == Terrain::Water {
        i.line(
            -0.5,
            -0.42,
            0.5,
            -0.42,
            2.5 * s,
            0.65 * s,
            rgb(203, 216, 191),
        );
        i.line(-0.5, 0.42, 0.5, 0.42, 2.5 * s, 0.65 * s, rgb(203, 216, 191));
    }
    if !v.calm && city.ext.weather.rain > 30 && nightratio(v) > 0.4 {
        i.patch(0., 0., 0.45, 0.22, 0., alpha(rgb(151, 191, 182), 0.08));
    }
}

pub(super) fn underground_ground(city: &City, index: usize, i: Iso) {
    let elevation = city
        .ext
        .visual_tile(index)
        .map(|c| c.elevation)
        .unwrap_or(0);
    i.diamond(tint(rgb(66, 89, 87), 1. + f32::from(elevation) / 900.));
    i.edge(alpha(rgb(131, 155, 136), 0.11), 0.35);
}
pub(super) fn underground_network(city: &City, index: usize, i: Iso, _v: View) {
    let colors = [
        rgb(240, 198, 99),
        rgb(101, 193, 216),
        rgb(174, 151, 119),
        rgb(184, 144, 217),
    ];
    let s = i.w / 12.;
    let masks = city.underground_connections(index);
    for (network, &mask) in masks.iter().enumerate() {
        if mask == 0 {
            continue;
        }
        let offset = (network as f32 - 1.5) * 0.055;
        let color = colors[network];
        for (bit, u, z) in [(1, 0.5, 0.), (2, 0., 0.5), (4, -0.5, 0.), (8, 0., -0.5)] {
            if mask & bit != 0 {
                i.line(
                    offset,
                    offset,
                    u + offset,
                    z + offset,
                    1. * s,
                    if network == 3 { 1.8 * s } else { 1.3 * s },
                    tint(color, 0.55),
                );
                i.line(
                    offset,
                    offset,
                    u + offset,
                    z + offset,
                    1.4 * s,
                    if network == 3 { 1. * s } else { 0.65 * s },
                    color,
                );
            }
        }
        let p = i.p(offset, offset, 1.5 * s);
        draw_circle(p.x, p.y, 1.1 * s, color);
    }
    if matches!(city.tiles[index].kind, Kind::SubwayStation | Kind::Station) {
        ring(i, 0., 0., 2. * s, 0.25, rgb(153, 189, 201));
    }
}

pub(super) fn vehicle_order(city: &City) -> Vec<(u32, usize)> {
    let mut order: Vec<_> = city
        .ext
        .transport
        .vehicles
        .iter()
        .enumerate()
        .map(|(i, v)| (v.tile, i))
        .collect();
    order.sort_unstable_by_key(|(tile, _)| *tile);
    order
}
pub(super) fn vehicles_at(city: &City, index: usize, view: View, order: &[(u32, usize)]) {
    use crate::transport::VehicleKind;
    let first = order.partition_point(|(tile, _)| *tile < index as u32);
    let width = usize::from(city.width);
    let w = crate::projection::tile_size(view.zoom).0 as f32;
    let s = w / 12.;
    for &(_, id) in order[first..]
        .iter()
        .take_while(|(tile, _)| *tile == index as u32)
    {
        let v = &city.ext.transport.vehicles[id];
        if view.underground != (v.kind == VehicleKind::Metro) {
            continue;
        }
        if v.tile as usize >= city.tiles.len() || v.next_tile as usize >= city.tiles.len() {
            continue;
        }
        let a = project_cell(
            city,
            (v.tile as usize % width) as i32,
            (v.tile as usize / width) as i32,
            view,
        );
        let b = project_cell(
            city,
            (v.next_tile as usize % width) as i32,
            (v.next_tile as usize / width) as i32,
            view,
        );
        let progress = f32::from(v.progress.min(1000)) / 1000.;
        let p = vec2(
            a.0 as f32 + (b.0 - a.0) as f32 * progress,
            a.1 as f32 + (b.1 - a.1) as f32 * progress,
        );
        if p.x < -25. || p.x > 825. || p.y < -25. || p.y > 475. {
            continue;
        }
        let lane = if matches!(
            v.kind,
            VehicleKind::Car
                | VehicleKind::Bus
                | VehicleKind::Truck
                | VehicleKind::Service
                | VehicleKind::Pedestrian
        ) {
            let offset = if v.kind == VehicleKind::Pedestrian {
                0.36
            } else {
                0.12
            };
            match v.heading % 4 {
                0 => [0., offset],
                1 => [-offset, 0.],
                2 => [0., -offset],
                _ => [offset, 0.],
            }
        } else {
            [0., 0.]
        };
        let iso = Iso {
            x: p.x + (lane[0] - lane[1]) * w,
            y: p.y + (lane[0] + lane[1]) * w / 2.,
            w,
            h: w / 2.,
        };
        let color = v
            .line_id
            .and_then(|id| city.ext.transport.lines.iter().find(|l| l.id == id))
            .map(|l| rgb(l.color[0], l.color[1], l.color[2]))
            .unwrap_or(match v.id % 5 {
                0 => rgb(228, 188, 113),
                1 => rgb(193, 123, 111),
                2 => rgb(213, 224, 207),
                3 => rgb(110, 178, 190),
                _ => rgb(127, 154, 177),
            });
        let along_x = v.heading.is_multiple_of(2);
        let service = if v.kind == VehicleKind::Service {
            city.ext
                .services
                .missions
                .iter()
                .find(|m| m.vehicle == v.id)
                .map(|m| m.purpose)
        } else {
            None
        };
        let (length, height) = match v.kind {
            VehicleKind::Bus => (0.48, 3.),
            VehicleKind::Train | VehicleKind::Metro => (0.8, 3.7),
            VehicleKind::Truck => (0.34, 3.2),
            VehicleKind::Service => {
                if service == Some(crate::services::ServicePurpose::Waste) {
                    (0.42, 3.4)
                } else {
                    (0.3, 2.6)
                }
            }
            _ => (0.23, 1.9),
        };
        match v.kind {
            VehicleKind::Aircraft | VehicleKind::Jet => {
                let p = p + vec2(0., -11. * s);
                draw_ellipse(
                    p.x + 4. * s,
                    p.y + 11. * s,
                    4. * s,
                    1.3 * s,
                    0.,
                    alpha(rgb(41, 79, 79), 0.18),
                );
                draw_line(
                    p.x - 4. * s,
                    p.y,
                    p.x + 4. * s,
                    p.y,
                    1.8 * s,
                    rgb(229, 236, 218),
                );
                draw_triangle(
                    p + vec2(-s, -4. * s),
                    p + vec2(2. * s, 4. * s),
                    p + vec2(-2. * s, 3. * s),
                    rgb(216, 229, 215),
                );
                draw_circle(p.x + 3.3 * s, p.y, 0.7 * s, color);
            }
            VehicleKind::Helicopter => {
                let q = p + vec2(0., -15. * s);
                draw_ellipse(
                    q.x + 3. * s,
                    q.y + 15. * s,
                    3.3 * s,
                    1.2 * s,
                    0.,
                    alpha(rgb(47, 78, 79), 0.18),
                );
                draw_ellipse(q.x, q.y, 3.5 * s, 1.8 * s, 0., rgb(215, 226, 207));
                draw_line(q.x - 2. * s, q.y, q.x - 7. * s, q.y - s, s, color);
                draw_line(
                    q.x - 7. * s,
                    q.y - 3. * s,
                    q.x - 7. * s,
                    q.y + s,
                    0.65 * s,
                    color,
                );
                let angle = if view.calm {
                    0.
                } else {
                    (view.tick % 60) as f32 * 0.33
                };
                let rotor = vec2(angle.cos() * 6. * s, angle.sin() * 1.4 * s);
                draw_line(
                    q.x - rotor.x,
                    q.y - 2.5 * s - rotor.y,
                    q.x + rotor.x,
                    q.y - 2.5 * s + rotor.y,
                    0.65 * s,
                    rgb(86, 115, 115),
                );
                draw_circle(q.x + s, q.y, s, rgb(108, 168, 181));
            }
            VehicleKind::Pedestrian => {
                let q = vec2(iso.x, iso.y) + vec2(0., -s);
                draw_line(q.x, q.y, q.x, q.y - 2.2 * s, 0.9 * s, color);
                draw_circle(q.x, q.y - 2.9 * s, 0.7 * s, rgb(205, 178, 145));
                let stride = if view.calm {
                    0.4
                } else {
                    (progress * 9.).sin() * 0.6
                } * s;
                draw_line(
                    q.x,
                    q.y,
                    q.x - stride,
                    q.y + 1.1 * s,
                    0.55 * s,
                    rgb(86, 118, 120),
                );
                draw_line(
                    q.x,
                    q.y,
                    q.x + stride,
                    q.y + 1.1 * s,
                    0.55 * s,
                    rgb(86, 118, 120),
                );
            }
            VehicleKind::Ferry => {
                iso.block(0., 0., 0.5, 0.25, 2.5 * s, color);
                iso.raised(-0.03, 0., 0.27, 0.2, 2.5 * s, 2. * s, rgb(220, 230, 208));
                if !view.calm {
                    iso.line(
                        -0.42,
                        -0.14,
                        -0.55,
                        -0.17,
                        0.,
                        0.6 * s,
                        alpha(rgb(187, 231, 216), 0.6),
                    );
                    iso.line(
                        -0.42,
                        0.14,
                        -0.55,
                        0.17,
                        0.,
                        0.6 * s,
                        alpha(rgb(187, 231, 216), 0.6),
                    );
                }
            }
            _ => {
                let body = match service {
                    Some(crate::services::ServicePurpose::Fire) => rgb(205, 91, 73),
                    Some(crate::services::ServicePurpose::Medical) => rgb(232, 237, 220),
                    Some(crate::services::ServicePurpose::Police) => rgb(78, 128, 164),
                    Some(crate::services::ServicePurpose::Waste) => rgb(112, 153, 106),
                    Some(crate::services::ServicePurpose::Deathcare) => rgb(96, 112, 117),
                    Some(crate::services::ServicePurpose::Repair) => rgb(218, 172, 83),
                    None => color,
                };
                iso.block(
                    0.,
                    0.,
                    if along_x { length } else { 0.16 },
                    if along_x { 0.16 } else { length },
                    height * s,
                    body,
                );
                iso.patch(
                    0.,
                    0.,
                    if along_x { length * 0.65 } else { 0.12 },
                    if along_x { 0.12 } else { length * 0.65 },
                    (height + 0.2) * s,
                    rgb(86, 139, 151),
                );
                if matches!(
                    v.kind,
                    VehicleKind::Bus | VehicleKind::Train | VehicleKind::Metro
                ) {
                    for k in 0..3 {
                        let u = -length * 0.3 + k as f32 * length * 0.3;
                        if along_x {
                            iso.line(
                                u,
                                0.09,
                                u + 0.045,
                                0.09,
                                height * s * 0.7,
                                0.65 * s,
                                rgb(195, 223, 211),
                            );
                        } else {
                            iso.line(
                                0.09,
                                u,
                                0.09,
                                u + 0.045,
                                height * s * 0.7,
                                0.65 * s,
                                rgb(195, 223, 211),
                            );
                        }
                    }
                }
                if nightratio(view) > 0.2 {
                    let direction = if v.heading < 2 { 1. } else { -1. };
                    let head = if along_x {
                        iso.p(direction * length / 2., 0., height * s * 0.5)
                    } else {
                        iso.p(0., direction * length / 2., height * s * 0.5)
                    };
                    draw_circle(head.x, head.y, 0.55 * s, rgb(251, 227, 166));
                }
                if v.kind == VehicleKind::Service {
                    let emergency = matches!(
                        service,
                        Some(
                            crate::services::ServicePurpose::Fire
                                | crate::services::ServicePurpose::Medical
                                | crate::services::ServicePurpose::Police
                        )
                    );
                    let c = if !emergency {
                        rgb(244, 192, 92)
                    } else if view.calm || (view.tick / 12).is_multiple_of(2) {
                        rgb(222, 111, 94)
                    } else {
                        rgb(107, 181, 221)
                    };
                    let p = iso.p(0., 0., (height + 0.6) * s);
                    draw_rectangle(p.x - s, p.y - 0.5 * s, 2. * s, s, c);
                    if service == Some(crate::services::ServicePurpose::Medical) {
                        iso.line(
                            -0.06,
                            0.,
                            0.06,
                            0.,
                            (height + 0.4) * s,
                            0.65 * s,
                            rgb(203, 86, 74),
                        );
                        iso.line(
                            0.,
                            -0.06,
                            0.,
                            0.06,
                            (height + 0.4) * s,
                            0.65 * s,
                            rgb(203, 86, 74),
                        );
                    }
                }
            }
        }
    }
}

fn nightratio(v: View) -> f32 {
    if v.calm {
        return 0.;
    }
    // This saved-tick cycle shares the client's day/night ambience boundary.
    let tick = (v.tick % 7200) as f32;
    let t = if tick < 3900. {
        0.
    } else if tick < 4500. {
        (tick - 3900.) / 600.
    } else if tick < 6600. {
        1.
    } else {
        (7200. - tick) / 600.
    };
    t * t * (3. - 2. * t)
}

pub(super) fn atmosphere(city: &City, v: View) {
    if v.underground || v.overlay > 0 {
        return;
    }
    let night = nightratio(v);
    if night > 0.01 {
        draw_rectangle(
            0.,
            0.,
            800.,
            450.,
            Color::new(0.035, 0.075, 0.16, night * 0.67),
        );
    }
    let cloud = f32::from(city.ext.weather.cloud.min(100)) / 100.;
    if cloud > 0.25 {
        draw_rectangle(
            0.,
            0.,
            800.,
            450.,
            Color::new(0.13, 0.22, 0.29, cloud * 0.11),
        );
    }
}
pub(super) fn night_lights(city: &City, index: usize, i: Iso, n: u32, v: View) {
    if v.overlay > 0 {
        return;
    }
    let night = nightratio(v);
    if night < 0.05 {
        return;
    }
    let t = &city.tiles[index];
    let s = i.w / 12.;
    let inactive = city
        .ext
        .visual_tile(index)
        .is_some_and(|c| c.abandoned || c.rubble || c.construction < 100);
    if inactive {
        return;
    }
    if street(t.kind) && t.powered && index.is_multiple_of(3) {
        let p = i.p(0.35, 0.32, 0.);
        draw_ellipse(
            p.x,
            p.y,
            4.3 * s,
            2.1 * s,
            0.,
            alpha(rgb(248, 212, 134), night * 0.12),
        );
        draw_line(p.x, p.y, p.x, p.y - 6. * s, 0.6 * s, rgb(142, 170, 165));
        draw_circle(p.x, p.y - 6. * s, 1.1 * s, alpha(rgb(255, 225, 164), night));
    }
    if t.powered && t.kind.is_building() && (!t.kind.is_zone() || t.level > 0) {
        let height = match t.kind {
            Kind::Residential if t.level == 1 => 6.,
            Kind::Residential => 9. + f32::from(t.level) * 4. + (n % 3) as f32 * 2.,
            Kind::Commercial if t.level == 1 => 6.,
            Kind::Commercial => 10. + f32::from(t.level) * 7. + (n % 4) as f32 * 2.,
            Kind::Hotel => 23.,
            Kind::ClockTower => 23.,
            Kind::Arcology => 36.,
            _ => 8.,
        };
        let rows = (height / 4.).floor() as u32;
        for row in 0..rows {
            for col in 0..3 {
                if !n.wrapping_add(row * 5 + col * 3).is_multiple_of(3) {
                    continue;
                }
                let u = -0.23 + col as f32 * 0.23;
                let z = (2.4 + row as f32 * 4.) * s;
                let a = i.p(u, 0.355, z);
                let b = i.p(0.355, u, z);
                let c = alpha(rgb(251, 216, 146), night * 0.9);
                draw_line(
                    a.x - 0.65 * s,
                    a.y - 0.32 * s,
                    a.x + 0.65 * s,
                    a.y + 0.32 * s,
                    s,
                    c,
                );
                draw_line(
                    b.x - 0.65 * s,
                    b.y + 0.32 * s,
                    b.x + 0.65 * s,
                    b.y - 0.32 * s,
                    s,
                    tint(c, 0.85),
                );
            }
        }
    }
}
pub(super) fn weather(city: &City, v: View) {
    use crate::world::WeatherKind;
    if v.calm || v.overlay > 0 {
        return;
    }
    let weather = &city.ext.weather;
    // Cool, saturated air produces low mist; its amount and drift follow the
    // saved weather and tick rather than wall-clock randomness.
    let mist = (i16::from(weather.cloud) - 48).max(0) as f32 / 52.
        * (18 - weather.temperature).clamp(0, 18) as f32
        / 18.;
    if mist > 0.02 {
        for k in 0..11 {
            let n = noise(k, 719);
            let x = ((n % 940) as f32 + v.tick as f32 * f32::from(weather.wind_x) * 0.006)
                .rem_euclid(1020.)
                - 110.;
            let y = 90. + ((n >> 12) % 330) as f32;
            draw_ellipse(
                x,
                y,
                110. + (n % 75) as f32,
                14. + ((n >> 8) % 15) as f32,
                0.,
                alpha(rgb(217, 232, 223), mist * 0.065),
            );
        }
    }
    let rain = matches!(weather.kind, WeatherKind::Rain | WeatherKind::Storm);
    let snow = weather.kind == WeatherKind::Snow;
    if !rain && !snow {
        return;
    }
    let count = if snow {
        72
    } else {
        usize::from(weather.rain).min(110) + 20
    };
    let tick = v.tick as f32;
    for k in 0..count {
        let n = noise(k as i32, 701);
        let speed = if snow { 0.22 } else { 2.5 };
        let x = ((n % 809) as f32
            + tick * (f32::from(weather.wind_x) - f32::from(weather.wind_y)) * 0.018)
            .rem_euclid(820.)
            - 10.;
        let y = (((n >> 12) % 463) as f32 + tick * speed).rem_euclid(470.) - 10.;
        if snow {
            draw_circle(
                x,
                y,
                0.65 + (n % 3) as f32 * 0.18,
                alpha(rgb(234, 243, 233), 0.62),
            );
        } else {
            draw_line(
                x,
                y,
                x + (f32::from(weather.wind_x) - f32::from(weather.wind_y)) * 0.08,
                y + 4.5,
                0.5,
                alpha(rgb(184, 216, 219), 0.34),
            );
        }
    }
}

pub(super) fn events(city: &City, v: View) {
    use crate::civic::Hazard;
    let width = usize::from(city.width);
    let s = crate::projection::tile_size(v.zoom).0 as f32 / 12.;
    for event in &city.civic.active_events {
        if city.civic.day > event.until_day || event.tile as usize >= city.tiles.len() {
            continue;
        }
        let tile = event.tile as usize;
        let (x, y) = project_cell(city, (tile % width) as i32, (tile / width) as i32, v);
        let p = vec2(x as f32, y as f32);
        if p.x < -100. || p.x > 900. || p.y < -120. || p.y > 550. {
            continue;
        }
        let phase = if v.calm {
            0.5
        } else {
            (v.tick % 180) as f32 / 180.
        };
        match event.kind {
            Hazard::Earthquake => {
                for k in 0..4 {
                    let d = k as f32 * 3. * s;
                    draw_line(
                        p.x - d,
                        p.y + d * 0.3,
                        p.x + d,
                        p.y - d * 0.1,
                        0.8 * s,
                        alpha(rgb(102, 114, 89), 0.8),
                    );
                }
            }
            Hazard::Flood => {
                draw_ellipse(p.x, p.y, 18. * s, 7. * s, 0., alpha(rgb(85, 173, 193), 0.4));
                for k in 0..3 {
                    draw_line(
                        p.x - 12. * s,
                        p.y + k as f32 * 2. * s,
                        p.x + 12. * s,
                        p.y + k as f32 * 2. * s,
                        0.6 * s,
                        alpha(rgb(186, 229, 215), 0.55),
                    );
                }
            }
            Hazard::Meteor => {
                let q = p + vec2((1. - phase) * 45. * s, -(1. - phase) * 70. * s);
                draw_line(
                    q.x,
                    q.y,
                    q.x + 16. * s,
                    q.y - 22. * s,
                    4. * s,
                    alpha(rgb(235, 166, 98), 0.38),
                );
                draw_poly(q.x, q.y, 7, 4. * s, 15., rgb(156, 133, 105));
                draw_circle_lines(p.x, p.y, 5. * s, 1.2 * s, rgb(157, 133, 99));
            }
            Hazard::Tornado => {
                for k in 0..7 {
                    let h = k as f32 * 4. * s;
                    let sway = ((phase * 6. + k as f32) * 2.).sin() * 2. * s;
                    draw_ellipse(
                        p.x + sway,
                        p.y - h,
                        (1.5 + k as f32 * 0.8) * s,
                        1.5 * s,
                        0.,
                        alpha(rgb(135, 161, 150), 0.45),
                    );
                }
            }
            Hazard::HostileUfo | Hazard::FriendlyUfo => {
                let q = p + vec2(0., -29. * s);
                if event.kind == Hazard::HostileUfo {
                    draw_triangle(
                        q,
                        p + vec2(-10. * s, 2. * s),
                        p + vec2(10. * s, 2. * s),
                        alpha(rgb(143, 217, 198), 0.18),
                    );
                }
                draw_ellipse(q.x, q.y, 11. * s, 3.2 * s, 0., rgb(151, 192, 186));
                draw_ellipse(q.x, q.y - 2. * s, 5. * s, 3. * s, 0., rgb(183, 226, 217));
                for k in 0..4 {
                    draw_circle(
                        q.x + (-6. + k as f32 * 4.) * s,
                        q.y + 0.5 * s,
                        s,
                        if event.kind == Hazard::FriendlyUfo {
                            rgb(237, 211, 139)
                        } else {
                            rgb(151, 226, 195)
                        },
                    );
                }
            }
            Hazard::PlaneCrash => {
                draw_line(
                    p.x - 7. * s,
                    p.y - 2. * s,
                    p.x + 5. * s,
                    p.y + 3. * s,
                    2. * s,
                    rgb(192, 204, 182),
                );
                draw_triangle(
                    p + vec2(-3. * s, -6. * s),
                    p + vec2(2. * s, 5. * s),
                    p + vec2(-5. * s, 2. * s),
                    rgb(180, 195, 178),
                );
                let iso = Iso {
                    x: p.x,
                    y: p.y,
                    w: s * 12.,
                    h: s * 6.,
                };
                fire(
                    city,
                    iso,
                    event.intensity.min(100),
                    if v.calm { 0 } else { v.tick },
                    event.tile,
                );
            }
            Hazard::Monster => {
                let q = p + vec2(0., -10. * s);
                draw_ellipse(q.x, q.y, 6. * s, 10. * s, 0., rgb(89, 139, 107));
                draw_circle(q.x, q.y - 9. * s, 4. * s, rgb(104, 155, 114));
                draw_triangle(
                    q + vec2(-5. * s, 3. * s),
                    q + vec2(-13. * s, 8. * s),
                    q + vec2(-4. * s, -2. * s),
                    rgb(79, 126, 100),
                );
                draw_circle(q.x + 2. * s, q.y - 10. * s, 0.7 * s, rgb(235, 216, 144));
            }
            Hazard::Riot => {
                for k in 0..8 {
                    let d = k as f32;
                    let x = p.x + (d % 4. - 1.5) * 3. * s;
                    let y = p.y + (d / 4.).floor() * 2. * s;
                    draw_line(
                        x,
                        y,
                        x,
                        y - 3. * s,
                        1.1 * s,
                        rgb(109 + (k % 3) as u8 * 25, 137, 127),
                    );
                    draw_circle(x, y - 4. * s, 0.9 * s, rgb(203, 178, 139));
                    if k % 3 == 0 {
                        draw_line(x, y - 4. * s, x, y - 7. * s, 0.5 * s, rgb(163, 174, 146));
                        draw_rectangle(x, y - 8. * s, 3. * s, 2. * s, rgb(217, 199, 148));
                    }
                }
            }
            Hazard::Meltdown => {
                draw_circle_lines(
                    p.x,
                    p.y - 9. * s,
                    8. * s,
                    1.3 * s,
                    alpha(rgb(177, 211, 113), 0.8),
                );
                for k in 0..3 {
                    let a = k as f32 * std::f32::consts::TAU / 3.;
                    let q = p + vec2(a.cos(), a.sin()) * 5. * s + vec2(0., -9. * s);
                    draw_circle(q.x, q.y, 2. * s, alpha(rgb(160, 197, 102), 0.75));
                }
            }
        }
    }
    if v.calm {
        return;
    }
    for c in &city.civic.celebrations {
        if city.civic.day < c.started_day
            || city.civic.day > c.until_day
            || c.tile as usize >= city.tiles.len()
        {
            continue;
        }
        let tile = c.tile as usize;
        let (x, y) = project_cell(city, (tile % width) as i32, (tile / width) as i32, v);
        let origin = c
            .milestone
            .map_or(u32::from(c.project), |id| u32::from(id) + 4);
        let colors = match origin % 4 {
            0 => [rgb(242, 200, 118), rgb(136, 218, 214)],
            1 => [rgb(240, 151, 117), rgb(180, 226, 154)],
            2 => [rgb(191, 170, 235), rgb(246, 216, 139)],
            _ => [rgb(236, 163, 194), rgb(136, 207, 235)],
        };
        let phase = (v.tick.wrapping_add(c.tile).wrapping_add(origin * 17) % 120) as f32 / 120.;
        let radius = (2. + phase * 12.) * s;
        let center = vec2(x as f32, y as f32 - 26. * s);
        for k in 0..12 {
            let a = k as f32 * std::f32::consts::TAU / 12.;
            let dir = vec2(a.cos(), a.sin());
            let from = center + dir * radius * 0.72;
            let to = center + dir * radius;
            draw_line(
                from.x,
                from.y,
                to.x,
                to.y,
                0.8 * s,
                alpha(colors[((k + origin) % 2) as usize], 1. - phase),
            );
        }
    }
}

pub(super) fn building_eras(city: &City) -> Vec<u8> {
    let mut eras = vec![0; city.tiles.len()];
    for b in &city.ext.buildings {
        if let Some(era) = eras.get_mut(b.tile as usize) {
            *era = b.era;
        }
    }
    eras
}

fn building_height(t: &Tile, n: u32) -> f32 {
    match t.kind {
        Kind::Residential if t.level == 1 => 10.,
        Kind::Residential => 9. + f32::from(t.level) * 4. + (n % 3) as f32 * 2.,
        Kind::Commercial if t.level == 1 => 6.,
        Kind::Commercial => 10. + f32::from(t.level) * 7. + (n % 4) as f32 * 2.,
        Kind::Industrial if n.is_multiple_of(3) || t.level >= 3 => {
            12. + f32::from(t.level) * 2. + (n % 4) as f32
        }
        Kind::Industrial => 6. + f32::from(t.level) * 1.5,
        Kind::Coal => 26.,
        Kind::Nuclear => 18.,
        Kind::Wind => 31.,
        Kind::WaterTower => 18.,
        Kind::ClockTower => 27.,
        Kind::Arcology => 43.,
        Kind::Hotel => 25.,
        Kind::Spaceport => 27.,
        Kind::WorldExpo => 17.,
        Kind::Monument => 21.,
        Kind::Prison => 15.,
        Kind::FireHQ => 19.,
        Kind::Observatory => 18.,
        Kind::ScienceCenter => 17.,
        Kind::CityHall => 18.,
        Kind::Hospital => 14.,
        Kind::University => 21.,
        Kind::Airport | Kind::Airfield => 17.,
        Kind::Incinerator => 27.,
        Kind::Gas => 21.,
        _ => 10.,
    }
}
/// Status marks use the same utility/access/lifecycle state as the simulation.
pub(super) fn problems(city: &City, index: usize, i: Iso, n: u32, v: View) {
    if v.photo || v.underground || v.overlay > 0 {
        return;
    }
    let t = &city.tiles[index];
    if !t.kind.is_building() || (t.kind.is_zone() && t.level == 0) {
        return;
    }
    let cell = city.ext.visual_tile(index);
    if cell.is_some_and(|c| c.construction < 100 || c.rubble) {
        return;
    }
    let abandoned = cell.is_some_and(|c| c.abandoned);
    let issues = [
        (4, t.fire > 0),
        (3, abandoned),
        (0, !t.connected),
        (1, !t.powered),
        (2, !t.watered),
    ];
    let s = i.w / 12.;
    let count = issues.iter().filter(|(_, active)| *active).count().min(3);
    let pulse = if v.calm {
        0.95
    } else {
        0.91 + ((v.tick % 180) as f32 * 0.075).sin() * 0.07
    };
    for (k, &(issue, _)) in issues
        .iter()
        .filter(|(_, active)| *active)
        .take(3)
        .enumerate()
    {
        let p = i.p(0., 0., (building_height(t, n) + 4.) * s)
            + vec2((k as f32 - (count as f32 - 1.) / 2.) * 5.4 * s, 0.);
        let color = if issue == 4 {
            rgb(228, 132, 94)
        } else if issue == 3 {
            rgb(188, 163, 124)
        } else {
            rgb(242, 203, 126)
        };
        draw_circle(p.x, p.y, 2.5 * s, alpha(rgb(42, 77, 85), pulse));
        draw_circle_lines(p.x, p.y, 2.5 * s, 0.45 * s, alpha(color, pulse));
        match issue {
            0 => {
                draw_line(p.x - 1.4 * s, p.y, p.x + 1.4 * s, p.y, 0.6 * s, color);
                for x in [-1.4, 1.4] {
                    draw_circle(p.x + x * s, p.y, 0.65 * s, color);
                }
                draw_line(
                    p.x - 0.4 * s,
                    p.y + 1.2 * s,
                    p.x + 0.5 * s,
                    p.y - 1.2 * s,
                    0.65 * s,
                    rgb(231, 226, 193),
                );
            }
            1 => {
                draw_triangle(
                    p + vec2(0.5 * s, -1.7 * s),
                    p + vec2(-1.1 * s, 0.3 * s),
                    p + vec2(0.2 * s, 0.3 * s),
                    color,
                );
                draw_triangle(
                    p + vec2(-0.3 * s, 1.7 * s),
                    p + vec2(1.2 * s, -0.3 * s),
                    p + vec2(-0.1 * s, -0.3 * s),
                    color,
                );
            }
            2 => {
                draw_triangle(
                    p + vec2(0., -1.7 * s),
                    p + vec2(-1.1 * s, 0.3 * s),
                    p + vec2(1.1 * s, 0.3 * s),
                    rgb(124, 207, 225),
                );
                draw_circle(p.x, p.y + 0.5 * s, s, rgb(124, 207, 225));
            }
            3 => {
                draw_line(
                    p.x - 1.4 * s,
                    p.y + 1.4 * s,
                    p.x + 1.4 * s,
                    p.y - 1.4 * s,
                    0.8 * s,
                    color,
                );
                draw_line(
                    p.x - 1.4 * s,
                    p.y - 1.4 * s,
                    p.x + 1.4 * s,
                    p.y + 1.4 * s,
                    0.8 * s,
                    color,
                );
            }
            _ => {
                draw_triangle(
                    p + vec2(0., -1.7 * s),
                    p + vec2(-1.3 * s, 1.2 * s),
                    p + vec2(1.3 * s, 1.2 * s),
                    color,
                );
                draw_circle(p.x, p.y + 0.5 * s, 0.6 * s, rgb(252, 222, 151));
            }
        }
    }
}

pub(super) fn rail_detail(city: &City, index: usize, i: Iso) {
    if !city.tiles[index].powered || !index.is_multiple_of(4) {
        return;
    }
    let s = i.w / 12.;
    let p = i.p(-0.35, -0.28, 0.);
    let top = p + vec2(0., -12. * s);
    draw_line(p.x, p.y, top.x, top.y, 0.8 * s, rgb(140, 164, 153));
    draw_line(
        top.x,
        top.y,
        top.x + 7. * s,
        top.y + 2. * s,
        0.7 * s,
        rgb(183, 196, 169),
    );
    let w = usize::from(city.width);
    let x = index % w;
    let y = index / w;
    for (dx, dy) in [(1, 0), (0, 1)] {
        if city.tile(x as i32 + dx, y as i32 + dy).is_some_and(|t| {
            matches!(t.kind, Kind::Rail | Kind::Station | Kind::FreightStation) || t.rail_crossing
        }) {
            let q = i.p(dx as f32, dy as f32, 11. * s);
            let a = i.p(0., 0., 11. * s);
            draw_line(a.x, a.y, q.x, q.y, 0.4 * s, rgb(74, 109, 109));
        }
    }
}
