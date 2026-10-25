//! Pseudo-3D rendering: parallax background, the road drawn front to back, then sprites and
//! the player back to front.
//!
//! Layers keep the passes ordered. The road is drawn as plain filled quads, which keep their
//! call order within the road layer, so farther segments paint over nearer ones like on the
//! original's canvas.

use game_toolkit::prelude::*;

use crate::ecs::MainWorldStates;
use crate::race::*;
use crate::track::*;

const BACKGROUND_LAYER: i16 = 0;
const ROAD_LAYER: i16 = 1;
const SPRITE_LAYER: i16 = 2;
pub const HUD_LAYER: i16 = 3;

pub struct Textures {
    pub background: TextureId,
    pub sprites: TextureId,
}

/// A track point projected onto the screen.
#[derive(Debug, Clone, Copy, Default)]
struct Projected {
    x: f32,
    y: f32,
    w: f32,
    scale: f32,
    camera_y: f32,
    camera_z: f32,
}

/// The projection of one visible segment.
#[derive(Debug, Clone, Copy, Default)]
struct SegmentView {
    p1: Projected,
    p2: Projected,
    /// Screen y above which the hills in front have already covered the screen.
    clip: f32,
}

/// Renders a frame, reusing `views` as per-frame scratch space.
pub struct Renderer {
    views: Vec<SegmentView>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            views: vec![SegmentView::default(); DRAW_DISTANCE],
        }
    }

    pub fn draw(
        &mut self,
        p: &mut Painter<'_>,
        tex: &Textures,
        states: &MainWorldStates,
        size: [f32; 2],
        rng: &mut Rng,
    ) {
        let [width, height] = size;
        let track = &states.track;
        let player = &states.player_view;
        let camera_depth = camera_depth();
        let player_z = player_z();
        let resolution = height / 480.0;

        let base = track.find(player.position);
        let base_percent = percent_remaining(player.position, SEGMENT_LENGTH);
        let player_segment = track.find(player.position + player_z);
        let player_percent = percent_remaining(player.position + player_z, SEGMENT_LENGTH);
        let ps = &track.segments[player_segment];
        let player_y = interpolate(ps.p1[0], ps.p2[0], player_percent);

        p.clear(SKY_COLOR);
        let bg = tex.background;
        draw_backdrop(
            p,
            bg,
            size,
            SKY,
            states.backdrop.sky,
            resolution * SKY_SPEED * player_y,
        );
        draw_backdrop(
            p,
            bg,
            size,
            HILLS,
            states.backdrop.hills,
            resolution * HILL_SPEED * player_y,
        );
        draw_backdrop(
            p,
            bg,
            size,
            TREES,
            states.backdrop.trees,
            resolution * TREE_SPEED * player_y,
        );

        // Road, front to back. Curves bend the road by accumulating `dx` into `x`.
        let mut max_y = height;
        let mut x = 0.0;
        let mut dx = -(track.segments[base].curve * base_percent);
        for n in 0..DRAW_DISTANCE {
            let segment = &track.segments[(base + n) % track.len()];
            let looped = segment.index < base;
            let camera_z = player.position - if looped { track.length() } else { 0.0 };
            let camera_x = player.x * ROAD_WIDTH;
            let camera_y = player_y + CAMERA_HEIGHT;
            let project = |world: [f32; 2], camera_x: f32| {
                let camera_y = world[0] - camera_y;
                let camera_z = world[1] - camera_z;
                let scale = camera_depth / camera_z;
                Projected {
                    x: (width / 2.0 - scale * camera_x * width / 2.0).round(),
                    y: (height / 2.0 - scale * camera_y * height / 2.0).round(),
                    w: (scale * ROAD_WIDTH * width / 2.0).round(),
                    scale,
                    camera_y,
                    camera_z,
                }
            };
            let view = SegmentView {
                p1: project(segment.p1, camera_x - x),
                p2: project(segment.p2, camera_x - x - dx),
                clip: max_y,
            };
            self.views[n] = view;

            x += dx;
            dx += segment.curve;

            let behind_us = view.p1.camera_z <= camera_depth;
            let back_face = view.p2.y >= view.p1.y;
            let hidden_by_hill = view.p2.y >= max_y;
            if behind_us || back_face || hidden_by_hill {
                continue;
            }

            let fog = exponential_fog(n as f32 / DRAW_DISTANCE as f32, FOG_DENSITY);
            draw_segment(p, width, &view, fog, &segment.color);
            max_y = view.p1.y;
        }

        // Cars, scenery and the player, back to front.
        for n in (1..DRAW_DISTANCE).rev() {
            let segment = &track.segments[(base + n) % track.len()];
            let view = &self.views[n];

            for car in states.traffic_grid.cars(segment.index) {
                let scale = interpolate(view.p1.scale, view.p2.scale, car.percent);
                let x = interpolate(view.p1.x, view.p2.x, car.percent)
                    + scale * car.offset * ROAD_WIDTH * width / 2.0;
                let y = interpolate(view.p1.y, view.p2.y, car.percent);
                let anchor = [-0.5, -1.0];
                // The car art shows the rear and right flank, as seen when the car is to our
                // left; mirror it when the car is to our right.
                let mirror = x > width / 2.0;
                draw_sprite(
                    p,
                    tex,
                    width,
                    car.sprite,
                    scale,
                    [x, y],
                    anchor,
                    Some(view.clip),
                    mirror,
                );
            }

            for sprite in &segment.sprites {
                let scale = view.p1.scale;
                let x = view.p1.x + scale * sprite.offset * ROAD_WIDTH * width / 2.0;
                let anchor = [if sprite.offset < 0.0 { -1.0 } else { 0.0 }, -1.0];
                let pos = [x, view.p1.y];
                draw_sprite(
                    p,
                    tex,
                    width,
                    sprite.source,
                    scale,
                    pos,
                    anchor,
                    Some(view.clip),
                    false,
                );
            }

            if segment.index == player_segment {
                let camera_y = interpolate(view.p1.camera_y, view.p2.camera_y, player_percent);
                let scale = camera_depth / player_z;
                let y = height / 2.0 - scale * camera_y * height / 2.0;
                let bounce = 1.5 * rng.next_f32() * (player.speed / MAX_SPEED) * resolution;
                let bounce = bounce * random_side(rng);
                let sprite = player_sprite(player.steer, player.updown);
                let pos = [width / 2.0, y + bounce];
                draw_sprite(p, tex, width, sprite, scale, pos, [-0.5, -1.0], None, false);
            }
        }
    }
}

fn player_sprite(steer: f32, updown: f32) -> AtlasRect {
    use sprites::*;
    let uphill = updown > 0.0;
    match (steer.total_cmp(&0.0), uphill) {
        (std::cmp::Ordering::Less, true) => PLAYER_UPHILL_LEFT,
        (std::cmp::Ordering::Less, false) => PLAYER_LEFT,
        (std::cmp::Ordering::Greater, true) => PLAYER_UPHILL_RIGHT,
        (std::cmp::Ordering::Greater, false) => PLAYER_RIGHT,
        (_, true) => PLAYER_UPHILL_STRAIGHT,
        (_, false) => PLAYER_STRAIGHT,
    }
}

fn exponential_fog(distance: f32, density: f32) -> f32 {
    1.0 / (distance * distance * density).exp()
}

/// Draws the atlas region `src` (`x, y, w, h` in pixels) into the screen rect `dst`,
/// horizontally flipped if `mirror` is set.
fn blit(
    p: &mut Painter<'_>,
    tex: TextureId,
    src: AtlasRect,
    dst: [f32; 4],
    layer: i16,
    mirror: bool,
) {
    let inst = SpriteInstance::at([dst[0], dst[1]], [dst[2], dst[3]])
        .with_pixel_region([src.x, src.y], [src.w, src.h]);
    let inst = if mirror { inst.flip_x() } else { inst };
    p.sprite_ex(tex, inst, layer, BlendMode::Alpha);
}

/// A filled trapezoid between a horizontal span at `near` and one at `far` (`[x_center,
/// half_width, y]` each).
fn trapezoid(p: &mut Painter<'_>, near: [f32; 3], far: [f32; 3], color: [f32; 4]) {
    let [x1, w1, y1] = near;
    let [x2, w2, y2] = far;
    let corners = [[x1 - w1, y1], [x1 + w1, y1], [x2 + w2, y2], [x2 - w2, y2]];
    p.quad(corners, color, ROAD_LAYER);
}

/// One parallax background layer, scrolled horizontally by `rotation` (`0..1`) and
/// vertically by `offset` pixels. Shows half the layer's width, wrapping around its end.
fn draw_backdrop(
    p: &mut Painter<'_>,
    tex: TextureId,
    [width, height]: [f32; 2],
    layer: AtlasRect,
    rotation: f32,
    offset: f32,
) {
    let image_w = layer.w / 2.0;
    let source_x = layer.x + (layer.w * rotation).floor();
    let source_w = image_w.min(layer.x + layer.w - source_x);
    let dest_w = (width * (source_w / image_w)).floor();

    let src = AtlasRect {
        x: source_x,
        w: source_w,
        ..layer
    };
    let dst = [0.0, offset, dest_w, height];
    blit(p, tex, src, dst, BACKGROUND_LAYER, false);
    if source_w < image_w {
        let src = AtlasRect {
            w: image_w - source_w,
            ..layer
        };
        let dst = [dest_w - 1.0, offset, width - dest_w, height];
        blit(p, tex, src, dst, BACKGROUND_LAYER, false);
    }
}

fn draw_segment(p: &mut Painter<'_>, width: f32, view: &SegmentView, fog: f32, color: &Palette) {
    let (near, far) = (&view.p1, &view.p2);
    // The full-width band of grass (and fog) behind the road.
    let half = width / 2.0;
    let (band_near, band_far) = ([half, half, near.y], [half, half, far.y]);
    trapezoid(p, band_near, band_far, color.grass);

    let rumble = |w: f32| w / (2.0 * LANES as f32).max(6.0);
    let lane_marker = |w: f32| w / (8.0 * LANES as f32).max(32.0);

    // The rumble strips are a wider trapezoid under the road.
    let edge = |pt: &Projected| [pt.x, pt.w + rumble(pt.w), pt.y];
    trapezoid(p, edge(near), edge(far), color.rumble);
    let road = |pt: &Projected| [pt.x, pt.w, pt.y];
    trapezoid(p, road(near), road(far), color.road);

    if let Some(lane) = color.lane {
        let marker = |pt: &Projected, k: usize| {
            let lane_w = pt.w * 2.0 / LANES as f32;
            [
                pt.x - pt.w + lane_w * k as f32,
                lane_marker(pt.w) / 2.0,
                pt.y,
            ]
        };
        for k in 1..LANES {
            trapezoid(p, marker(near, k), marker(far, k), lane);
        }
    }

    if fog < 1.0 {
        let [r, g, b, _] = FOG_COLOR;
        trapezoid(p, band_near, band_far, [r, g, b, 1.0 - fog]);
    }
}

/// Draws a sprite scaled by the projection, anchored by `anchor` (fractions of its size added
/// to `pos`), cut off below `clip_y` where hills in front hide it, and optionally mirrored.
#[allow(clippy::too_many_arguments)]
fn draw_sprite(
    p: &mut Painter<'_>,
    tex: &Textures,
    width: f32,
    sprite: AtlasRect,
    scale: f32,
    pos: [f32; 2],
    anchor: [f32; 2],
    clip_y: Option<f32>,
    mirror: bool,
) {
    let world = scale * width / 2.0 * (sprites::SCALE * ROAD_WIDTH);
    let dest_w = sprite.w * world;
    let dest_h = sprite.h * world;
    let dest_x = pos[0] + dest_w * anchor[0];
    let dest_y = pos[1] + dest_h * anchor[1];

    let clip_h = clip_y.map_or(0.0, |clip| (dest_y + dest_h - clip).max(0.0));
    if clip_h >= dest_h {
        return;
    }
    let src = AtlasRect {
        h: sprite.h - sprite.h * clip_h / dest_h,
        ..sprite
    };
    let dst = [dest_x, dest_y, dest_w, dest_h - clip_h];
    blit(p, tex.sprites, src, dst, SPRITE_LAYER, mirror);
}
