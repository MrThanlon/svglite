#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("./vg_lite.rs");

use std::{slice, mem::transmute, ffi::c_void, f64::consts::PI};
use usvg::{
    self,
    Node,
    NodeKind::{Group, Path, Image, Text},
    Paint::{Color, LinearGradient, RadialGradient, Pattern},
    Transform, Visibility,
    PathSegment::*, NodeExt, Stop, Units, Tree
};

struct VGLiteConfig {
    target: *mut vg_lite_buffer,
    fill_rule: vg_lite_fill_t,
    /// Not used
    blend: vg_lite_blend_t,
    quality: vg_lite_quality_t
}

#[repr(C)]
#[derive(Clone)]
pub struct svglite_svg {
    svg: *mut Tree
}

#[no_mangle]
extern "C" fn svglite_free(svg: svglite_svg) {
    if !svg.svg.is_null() {
        unsafe {Box::from_raw(svg.svg)};
    }
}

#[no_mangle]
extern "C" fn svglite_svg_from_data(data: *const u8, len: usize) -> svglite_svg {
    if let Ok(svg) = Tree::from_data(unsafe {slice::from_raw_parts(data, len)}, &usvg::Options::default()) {
        svglite_svg { svg: Box::into_raw(Box::new(svg)) }
    } else {
        svglite_svg { svg: std::ptr::null_mut() }
    }
}

#[no_mangle]
extern "C" fn svglite_render(
    target: &mut vg_lite_buffer,
    svg: svglite_svg,
    fill_rule: vg_lite_fill_t,
    blend: vg_lite_blend_t,
    quality: vg_lite_quality_t
) -> vg_lite_error {
    let svg = unsafe {std::ptr::read(svg.svg)};
    let mut error = unsafe { vg_lite_clear(target, transmute(0usize), 0x0000_0000) };
    if error != vg_lite_error_VG_LITE_SUCCESS {
        // println!("error: vg_lite_clear() failed with code {}", error);
        return error;
    }
    let mut viewbox_mat = Transform::default();
    viewbox_mat.scale(target.width as f64 / svg.view_box.rect.width(), target.height as f64 / svg.view_box.rect.height());
    viewbox_mat.translate(-svg.view_box.rect.left(), -svg.view_box.rect.top());
    error = dfs(&svg.root, &viewbox_mat, &VGLiteConfig {
        target,
        fill_rule,
        blend,
        quality
    });
    if error != vg_lite_error_VG_LITE_SUCCESS {
        // println!("error: dfs() failed with code {}", error);
        return error;
    }

    unsafe { vg_lite_finish() }
}

fn dfs(node: &Node, mat: &Transform, config: &VGLiteConfig) -> u32 {
    let mut m = mat.clone();
    m.append(&node.transform());
    match node.borrow().to_owned() {
        Group(_group) => {
            for child in node.children() {
                let e = dfs(&child, &m, config);
                if e != vg_lite_error_VG_LITE_SUCCESS {
                    return e;
                }
            }
        },
        Path(path) => {
            if path.visibility != Visibility::Visible {
                return vg_lite_error_VG_LITE_SUCCESS;
            }
            if let Some(fill) = path.fill {
                // build path
                let mut path_data = Vec::new();
                /* TODO: specify S32 format
                for seg in path.data.segments() {
                    match seg {
                        MoveTo { x, y } => {
                            path_data.push(2i32);
                            path_data.push(x.round() as i32);
                            path_data.push(y.round() as i32);
                        },
                        LineTo { x, y } => {
                            path_data.push(4i32);
                            path_data.push(x.round() as i32);
                            path_data.push(y.round() as i32);
                        },
                        CurveTo { x1, y1, x2, y2, x, y } => {
                            path_data.push(8i32);
                            path_data.push(x1.round() as i32);
                            path_data.push(y1.round() as i32);
                            path_data.push(x2.round() as i32);
                            path_data.push(y2.round() as i32);
                            path_data.push(x.round() as i32);
                            path_data.push(y.round() as i32);
                        },
                        ClosePath => {
                            path_data.push(0i32);
                        }
                    }
                }
                */
                for seg in path.data.segments() {
                    match seg {
                        MoveTo { x, y } => {
                            path_data.push(unsafe { transmute::<u32, f32>(2) });
                            path_data.push(x as f32);
                            path_data.push(y as f32);
                        },
                        LineTo { x, y } => {
                            path_data.push(unsafe { transmute::<u32, f32>(4) });
                            path_data.push(x as f32);
                            path_data.push(y as f32);
                        },
                        CurveTo { x1, y1, x2, y2, x, y } => {
                            path_data.push(unsafe { transmute::<u32, f32>(8) });
                            path_data.push(x1 as f32);
                            path_data.push(y1 as f32);
                            path_data.push(x2 as f32);
                            path_data.push(y2 as f32);
                            path_data.push(x as f32);
                            path_data.push(y as f32);
                        },
                        ClosePath => {
                            path_data.push(unsafe { transmute::<u32, f32>(0) });
                        }
                    }
                }
                let bbox = node.calculate_bbox().unwrap();
                let mut p = vg_lite_path {
                    bounding_box: [
                        bbox.x() as f32,
                        bbox.y() as f32,
                        (bbox.x() + bbox.width()) as f32,
                        (bbox.x() + bbox.width()) as f32
                    ],
                    quality: config.quality,
                    format: vg_lite_format_VG_LITE_FP32,
                    uploaded: unsafe { transmute::<[u32;8], vg_lite_hw_memory>([0;8]) },
                    path_length: (path_data.len() * 4) as i32,
                    path: path_data.as_mut_ptr() as *mut c_void,
                    path_changed: 1,
                    pdata_internal: 0
                };

                // let mut mat2 = mat.clone();
                // mat2.transform(mat);
                let mut mr = vg_lite_matrix {
                    m: [[m.a as f32, m.c as f32, m.e as f32],
                        [m.b as f32, m.d as f32, m.f as f32],
                        [0., 0., 1.]
                    ]
                };

                match fill.paint {
                    Color(color) => {
                        let c = ((fill.opacity.to_u8() as u32) << 24) |
                        ((color.red as u32) << 16) |
                        ((color.green as u32) << 8) |
                        ((color.blue as u32) << 0);

                        // TODO: return error
                        let error = unsafe { vg_lite_draw(
                            config.target,
                            &mut p,
                            config.fill_rule,
                            &mut mr,
                            config.blend,
                            c
                        ) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            return error;
                        }
                    },
                    LinearGradient(lg) => {
                        if lg.base.stops.len() > 16 {
                            println!("error: linearGradient stops must not bigger than 16");
                            return vg_lite_error_VG_LITE_NOT_SUPPORT;
                        }

                        // build gradient
                        let mut grad: vg_lite_linear_gradient = unsafe { transmute([0usize;53]) };
                        let mut error = unsafe { vg_lite_init_grad(&mut grad) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            return error;
                        }

                        let mut colors: Vec<vg_lite_color_t> = (&lg.base.stops).into_iter().map(|x| {
                            x.get_u32()
                        }).collect();

                        // stop is 0 to 255, use matrix to scale
                        let mut stops: Vec<u32> = (&lg.base.stops).into_iter().map(|x| {
                            x.offset.to_u8() as u32
                        }).collect();

                        let mat: *mut vg_lite_matrix;
                        unsafe {
                            vg_lite_set_grad(
                                &mut grad,
                                stops.len() as u32,
                                colors.as_mut_ptr(),
                                stops.as_mut_ptr()
                            );
                            vg_lite_update_grad(&mut grad);
                            mat = vg_lite_get_grad_matrix(&mut grad);
                        }
                        // FIXME: handle x1 x2 y1 y2
                        // Do transform
                        let mut grad_mat = lg.transform.clone();
                        let angle = {
                            if lg.x1 == lg.x2 {
                                if lg.y2 >= lg.y1 {
                                    90.
                                } else {
                                    -90.
                                }
                            } else {
                                (lg.y2 - lg.y1).atan2(lg.x2 - lg.x1) * 180. / PI
                            }
                        };
                        let s = (lg.x2 - lg.x1) / 255.;
                        if lg.base.units == Units::UserSpaceOnUse {
                            // FXIME
                            grad_mat.translate(lg.x1, lg.y1);
                            grad_mat.rotate(angle);
                            grad_mat.scale(s, s);
                        } else {
                            // original direction is from (0,0) to (1,0), now we need use x1 x2 y1 y2 to transform
                            grad_mat.rotate(angle);
                            let (sx, sy) = m.get_scale();
                            grad_mat.scale(bbox.width() * sx * s, bbox.height() * sy * s);
                            grad_mat.translate(lg.x1 / s, lg.y1 / s);
                        }

                        unsafe {
                            (*mat).m[0][0] = grad_mat.a as f32;
                            (*mat).m[0][1] = grad_mat.c as f32;
                            (*mat).m[0][2] = grad_mat.e as f32;
                            (*mat).m[1][0] = grad_mat.b as f32;
                            (*mat).m[1][1] = grad_mat.d as f32;
                            (*mat).m[1][2] = grad_mat.f as f32;
                            (*mat).m[2][0] = 0.;
                            (*mat).m[2][1] = 0.;
                            (*mat).m[2][2] = 1.;

                            error = vg_lite_draw_gradient(
                                config.target,
                                &mut p,
                                vg_lite_fill_VG_LITE_FILL_EVEN_ODD,
                                &mut mr,
                                &mut grad,
                                config.blend
                            );
                            if error != vg_lite_error_VG_LITE_SUCCESS {
                                return error;
                            }
                        };
                    },
                    RadialGradient(_rg) => {
                        // TODO
                    },
                    Pattern(_p) => {
                        // TODO
                    },
                }
            }
            if let Some(_stroke) = path.stroke {
                // stroke is not supported
            }
        },
        Image(_image) => {
            // TODO
        },
        Text(_text) => {
            // TODO
        }
    }
    return vg_lite_error_VG_LITE_SUCCESS;
}

trait U32Color {
    fn get_u32(&self) -> u32;
}

impl U32Color for Stop {
    fn get_u32(&self) -> u32 {
        ((self.opacity.to_u8() as u32) << 24) |
        ((self.color.red as u32) << 0) |
        ((self.color.green as u32) << 8) |
        ((self.color.blue as u32) << 16)
    }
}
