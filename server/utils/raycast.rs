// HELP FROM https://github.com/andyhall/fast-voxel-raycast/blob/master/index.js

use crate::libs::types::Coords3;

use super::math::approx_equals;

type GetVoxel = dyn Fn(i32, i32, i32) -> u32;

#[allow(clippy::too_many_arguments)]
fn trace_ray(
    get_voxel: &GetVoxel,
    px: f32,
    py: f32,
    pz: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    max_d: f32,
    hit_pos: &mut Coords3<f32>,
    hit_norm: &mut Coords3<i32>,
) -> u32 {
    let mut t = 0.0;

    let mut ix = px.floor() as i32;
    let mut iy = py.floor() as i32;
    let mut iz = pz.floor() as i32;

    let step_x = if dx > 0.0 { 1 } else { -1 };
    let step_y = if dy > 0.0 { 1 } else { -1 };
    let step_z = if dz > 0.0 { 1 } else { -1 };

    let tx_delta = (1.0 / dx).abs();
    let ty_delta = (1.0 / dy).abs();
    let tz_delta = (1.0 / dz).abs();

    let x_dist = if step_x > 0 {
        ix as f32 + 1.0 - px
    } else {
        px - ix as f32
    };
    let y_dist = if step_y > 0 {
        iy as f32 + 1.0 - py
    } else {
        py - iy as f32
    };
    let z_dist = if step_z > 0 {
        iz as f32 + 1.0 - pz
    } else {
        pz - iz as f32
    };

    let mut tx_max = if tx_delta < f32::MAX {
        tx_delta * x_dist
    } else {
        f32::MAX
    };
    let mut ty_max = if ty_delta < f32::MAX {
        ty_delta * y_dist
    } else {
        f32::MAX
    };
    let mut tz_max = if tz_delta < f32::MAX {
        tz_delta * z_dist
    } else {
        f32::MAX
    };

    let mut stepped_index = -1;

    #[allow(clippy::while_immutable_condition, clippy::collapsible_else_if)]
    while t <= max_d {
        // exit check
        let v = get_voxel(ix, iy, iz);
        if v != 0 {
            hit_pos.0 = px + t as f32 * dx;
            hit_pos.1 = py + t as f32 * dy;
            hit_pos.2 = pz + t as f32 * dz;

            hit_norm.0 = 0;
            hit_norm.1 = 0;
            hit_norm.2 = 0;

            if stepped_index == 0 {
                hit_norm.0 = -step_x;
            } else if stepped_index == 1 {
                hit_norm.1 = -step_y;
            } else if stepped_index == 2 {
                hit_norm.2 = -step_z;
            }

            return v;
        }

        if tx_max < ty_max {
            if tx_max < tz_max {
                ix += step_x;
                t = tx_max;
                tx_max += tx_delta;
                stepped_index = 0;
            } else {
                iz += step_z;
                t = tz_max;
                tz_max += tz_delta;
                stepped_index = 2;
            }
        } else {
            if ty_max < tz_max {
                iy += step_y;
                t = ty_max;
                ty_max += ty_delta;
                stepped_index = 1;
            } else {
                iz += step_z;
                t = tz_max;
                tz_max += tz_delta;
                stepped_index = 2;
            }
        }
    }

    // no voxel hit found
    hit_pos.0 = px + t * dx;
    hit_pos.1 = py + t * dy;
    hit_pos.2 = pz + t * dz;

    hit_norm.0 = 0;
    hit_norm.1 = 0;
    hit_norm.2 = 0;

    0
}

pub fn trace(
    max_d: f32,
    get_voxel: &GetVoxel,
    origin: &mut Coords3<f32>,
    direction: &mut Coords3<f32>,
    hit_pos: &mut Coords3<f32>,
    hit_norm: &mut Coords3<i32>,
) -> u32 {
    let Coords3(px, py, pz) = origin;
    let Coords3(dx, dy, dz) = direction;
    let ds = (*dx * *dx + *dy * *dy + *dz * *dz).sqrt();

    if approx_equals(&ds, &0.0) {
        // ?should return an error?
        panic!("Can't raycast along a zero vector");
    }

    *dx /= ds;
    *dy /= ds;
    *dz /= ds;

    trace_ray(
        get_voxel, *px, *py, *pz, *dx, *dy, *dz, max_d, hit_pos, hit_norm,
    )
}

#[cfg(test)]
mod tests {
    use log::debug;

    use super::*;

    #[test]
    fn sphere_test() {
        let voxel = |x: i32, y: i32, z: i32| {
            if x == 0 && y == 0 && z == 0 {
                1
            } else {
                0
            }
        };

        let mut hit_position = Coords3::default();
        let mut hit_normal = Coords3::default();

        // 2 * PI
        for t100 in (0..628).step_by(25) {
            let theta = t100 as f32 / 100.0;
            for p100 in (0..628).step_by(25) {
                let phi = p100 as f32 / 100.0;
                let mut dir = Coords3(theta.cos() * phi.cos(), theta.sin() * phi.cos(), phi.sin());
                for x1000 in (1..1000).step_by(250) {
                    let x = x1000 as f32 / 1000.0;
                    for y1000 in (1..1000).step_by(250) {
                        let y = y1000 as f32 / 1000.0;
                        for z1000 in (1..1000).step_by(250) {
                            let z = z1000 as f32 / 1000.0;
                            let mut pos =
                                Coords3(x - 2.0 * dir.0, y - 2.0 * dir.1, z - 2.0 * dir.2);
                            let b = trace(
                                10.0,
                                &voxel,
                                &mut pos,
                                &mut dir,
                                &mut hit_position,
                                &mut hit_normal,
                            );
                            assert_eq!(b, 1);
                        }
                    }
                }
            }
        }
    }
}