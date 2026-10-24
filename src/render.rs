//! Pseudo-3D rendering: parallax background, the road drawn front to back, then sprites and
//! the player back to front.
//!
//! The toolkit batches quads per layer, so the road trapezoids are filled as one-pixel-high
//! scanline rects. Layers keep the passes ordered; within a layer, draw order is preserved.

use game_toolkit::prelude::*;

use crate::race::*;
use crate::track::*;

const BACKGROUND_LAYER: i16 = 0;
const ROAD_LAYER: i16 = 1;
const SPRITE_LAYER: i16 = 2;
pub const HUD_LAYER: i16 = 3;

pub struct Textures {
    pub background: TextureId,
    pub sprites: TextureId,
    pub white: TextureId,
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
        race: &RaceState,
        size: [f32; 2],
        rng: &mut Rng,
    ) {
        let [width, height] = size;
        let track = &race.track;
        let player = &race.player;
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
            race.sky_offset,
            resolution * SKY_SPEED * player_y,
        );
        draw_backdrop(
            p,
            bg,
            size,
            HILLS,
            race.hill_offset,
            resolution * HILL_SPEED * player_y,
        );
        draw_backdrop(
            p,
            bg,
            size,
            TREES,
            race.tree_offset,
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
            draw_segment(p, tex.white, width, height, &view, fog, &segment.color);
            max_y = view.p1.y;
        }

        // Cars, scenery and the player, back to front.
        for n in (1..DRAW_DISTANCE).rev() {
            let segment = &track.segments[(base + n) % track.len()];
            let view = &self.views[n];

            for car in &race.traffic[segment.index] {
                let scale = interpolate(view.p1.scale, view.p2.scale, car.percent);
                let x = interpolate(view.p1.x, view.p2.x, car.percent)
                    + scale * car.offset * ROAD_WIDTH * width / 2.0;
                let y = interpolate(view.p1.y, view.p2.y, car.percent);
                let anchor = [-0.5, -1.0];
                draw_sprite(
                    p,
                    tex,
                    width,
                    car.sprite,
                    scale,
                    [x, y],
                    anchor,
                    Some(view.clip),
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
                draw_sprite(p, tex, width, sprite, scale, pos, [-0.5, -1.0], None);
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

/// Draws the atlas region `src` (`x, y, w, h` in pixels) into the screen rect `dst`.
fn blit(
    p: &mut Painter<'_>,
    tex: TextureId,
    atlas: [f32; 2],
    src: AtlasRect,
    dst: [f32; 4],
    layer: i16,
) {
    let [aw, ah] = atlas;
    let uv_min = [src.x / aw, src.y / ah];
    let uv_max = [(src.x + src.w) / aw, (src.y + src.h) / ah];
    let inst = SpriteInstance::at([dst[0], dst[1]], [dst[2], dst[3]]).with_uv(uv_min, uv_max);
    p.sprite_ex(tex, inst, layer, BlendMode::Alpha);
}

fn fill(p: &mut Painter<'_>, white: TextureId, dst: [f32; 4], color: [f32; 4]) {
    let inst = SpriteInstance::at([dst[0], dst[1]], [dst[2], dst[3]]).with_color(color);
    p.sprite_ex(white, inst, ROAD_LAYER, BlendMode::Alpha);
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
    blit(
        p,
        tex,
        BACKGROUND_ATLAS,
        src,
        [0.0, offset, dest_w, height],
        BACKGROUND_LAYER,
    );
    if source_w < image_w {
        let src = AtlasRect {
            w: image_w - source_w,
            ..layer
        };
        let dst = [dest_w - 1.0, offset, width - dest_w, height];
        blit(p, tex, BACKGROUND_ATLAS, src, dst, BACKGROUND_LAYER);
    }
}

fn draw_segment(
    p: &mut Painter<'_>,
    white: TextureId,
    width: f32,
    height: f32,
    view: &SegmentView,
    fog: f32,
    color: &Palette,
) {
    let (near, far) = (&view.p1, &view.p2);
    let top = far.y.max(0.0);
    let bottom = near.y.min(height);
    if bottom <= top {
        return;
    }
    fill(p, white, [0.0, top, width, bottom - top], color.grass);

    let rumble = |w: f32| w / (2.0 * LANES as f32).max(6.0);
    let lane_marker = |w: f32| w / (8.0 * LANES as f32).max(32.0);

    // Scanline-fill the trapezoids, sampling the edges at each row's center.
    let mut row = top.floor();
    while row < bottom {
        let t = ((row + 0.5 - far.y) / (near.y - far.y)).clamp(0.0, 1.0);
        let x = interpolate(far.x, near.x, t);
        let w = interpolate(far.w, near.w, t);
        let r = interpolate(rumble(far.w), rumble(near.w), t);
        fill(p, white, [x - w - r, row, 2.0 * (w + r), 1.0], color.rumble);
        fill(p, white, [x - w, row, 2.0 * w, 1.0], color.road);
        if let Some(lane) = color.lane {
            let l = interpolate(lane_marker(far.w), lane_marker(near.w), t);
            let lane_w = w * 2.0 / LANES as f32;
            for k in 1..LANES {
                let lane_x = x - w + lane_w * k as f32;
                fill(p, white, [lane_x - l / 2.0, row, l, 1.0], lane);
            }
        }
        row += 1.0;
    }

    if fog < 1.0 {
        let [r, g, b, _] = FOG_COLOR;
        fill(
            p,
            white,
            [0.0, top, width, bottom - top],
            [r, g, b, 1.0 - fog],
        );
    }
}

/// Draws a sprite scaled by the projection, anchored by `anchor` (fractions of its size added
/// to `pos`), and cut off below `clip_y` where hills in front hide it.
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
    blit(p, tex.sprites, SPRITE_ATLAS, src, dst, SPRITE_LAYER);
}
