#![allow(dead_code)]
// mod vglite;
mod vglite;

use vglite::*;
use std::{env::args, fs::{read_to_string, self}, mem::transmute, ffi::c_void, io::Write};
use usvg::{
    self,
    Node,
    NodeKind::{Group, Path, Image, Text},
    Paint::{Color, LinearGradient, RadialGradient, Pattern},
    Transform, Visibility,
    PathSegment::*, NodeExt, Stop
};

struct VGLiteConfig {
    target: *mut vg_lite_buffer,
    fill_rule: vg_lite_fill_t,
    /// Not used
    blend: vg_lite_blend_t,
    quality: vg_lite_quality_t
}

// FIXME: use RAII
fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 5 {
        println!("Usage: {} <width> <height> <Input SVG file> <Output PNG file>", args[0]);
        return;
    }

    // vglite
    let width: i32 = str::parse(args[1].as_str()).expect("error: width is not a valid number");
    let height: i32 = str::parse(args[2].as_str()).expect("error: height is not a valid number");
    let mut error = unsafe { vg_lite_init(width, height) };
    if error != vg_lite_error_VG_LITE_SUCCESS {
        println!("error: vg_lite_init() failed with code {}", error);
    }
    let mut buffer = vg_lite_buffer {
        width,
        height,
        stride: 0,
        tiled: 0,
        format: vg_lite_buffer_format_VG_LITE_RGBA8888,
        handle: unsafe { transmute(0usize) },
        memory: unsafe { transmute(0usize) },
        address: 0,
        yuv: unsafe { transmute([0;18]) },
        image_mode: 0,
        transparency_mode: 0,
        fc_enable: 0,
        fc_buffer: unsafe { transmute([0;30]) }
    };
    error = unsafe { vg_lite_allocate(&mut buffer) };
    if error != vg_lite_error_VG_LITE_SUCCESS {
        println!("error: vg_lite_allocate() failed with code {}", error);
        unsafe { vg_lite_close(); };
        return;
    }
    error = unsafe { vg_lite_clear(&mut buffer, transmute(0usize), 0xffff_ffff) };
    if error != vg_lite_error_VG_LITE_SUCCESS {
        println!("error: vg_lite_clear() failed with code {}", error);
        unsafe {
            vg_lite_free(&mut buffer);
            vg_lite_close();
        };
        return;
    }

    // svg
    if let Ok(svg) = read_to_string(args[3].as_str()) {
        if let Ok(svg) = usvg::Tree::from_str(svg.as_str(), &usvg::Options::default()) {
            let mut viewbox_mat = Transform::default();
            viewbox_mat.scale(width as f64 / svg.view_box.rect.width(), height as f64 / svg.view_box.rect.height());
            viewbox_mat.translate(-svg.view_box.rect.left(), -svg.view_box.rect.top());

            // dfs
            error = dfs(&svg.root, &viewbox_mat, &VGLiteConfig {
                target: &mut buffer,
                fill_rule: vg_lite_fill_VG_LITE_FILL_NON_ZERO,
                blend: vg_lite_blend_VG_LITE_BLEND_NONE,
                quality: vg_lite_quality_VG_LITE_HIGH
            });
            if error != vg_lite_error_VG_LITE_SUCCESS {
                println!("error: dfs() failed with code {}", error);
                unsafe {
                    vg_lite_free(&mut buffer);
                    vg_lite_close();
                };
                return;
            }

            error = unsafe { vg_lite_finish() };
            if error != vg_lite_error_VG_LITE_SUCCESS {
                println!("error: vg_lite_finish() failed with code {}", error);
                unsafe {
                    vg_lite_free(&mut buffer);
                    vg_lite_close();
                };
                return;
            }
            let data = unsafe {
                std::slice::from_raw_parts(buffer.memory as *const u8, (buffer.stride * buffer.height) as usize)
            };
            if let Ok(mut output) = fs::File::create(args[4].as_str()) {
                if let Err(_) = output.write(data) {
                    println!("error: write");
                }
            } else {
                println!("error: create output file");
            }
        }

    } else {
        println!("error: SVG file is not valid");   
    }

    unsafe {
        vg_lite_free(&mut buffer);
        vg_lite_close();
    };
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
                /*
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
                let mut m = vg_lite_matrix {
                    m: [[m.a as f32, m.c as f32, m.e as f32],
                        [m.b as f32, m.d as f32, m.f as f32],
                        [0., 0., 1.]
                    ]
                };

                match fill.paint {
                    Color(color) => {
                        println!("pure color");
                        let mut c = fill.opacity.to_u8() as u32;
                        c <<= 24;
                        c |= (color.red as u32) << 16;
                        c |= (color.green as u32) << 8;
                        c |= (color.blue as u32) << 0;

                        // TODO: return error
                        let error = unsafe { vg_lite_draw(
                            config.target,
                            &mut p,
                            config.fill_rule,
                            &mut m,
                            config.blend,
                            c
                        ) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            return error;
                        }
                    },
                    LinearGradient(lg) => {
                        println!("linearGradient");
                        if lg.base.stops.len() > 16 {
                            println!("error: linearGradient stops must not bigger than 16");
                            return vg_lite_error_VG_LITE_NOT_SUPPORT;
                        }
                        // FIXME: handle x1 x2 y1 y2
                        // build gradient
                        let mut grad: vg_lite_linear_gradient = unsafe { transmute([0usize;53]) };
                        let mut error = unsafe { vg_lite_init_grad(&mut grad) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            return error;
                        }

                        let mut colors: Vec<vg_lite_color_t> = (&lg.base.stops).into_iter().map(|x| {
                            x.get_u32()
                        }).collect();

                        let mut stops: Vec<u32> = (&lg.base.stops).into_iter().map(|x| {
                            x.offset.to_u8() as u32
                        }).collect();

                        let mat: *mut vg_lite_matrix;
                        unsafe {
                            vg_lite_set_grad(
                                &mut grad,
                                lg.base.stops.len() as u32,
                                colors.as_mut_ptr(),
                                stops.as_mut_ptr()
                            );
                            vg_lite_update_grad(&mut grad);
                            mat = vg_lite_get_grad_matrix(&mut grad);
                            vg_lite_identity(mat);
                            // TODO: do transform
                            error = vg_lite_draw_gradient(
                                config.target,
                                &mut p,
                                vg_lite_fill_VG_LITE_FILL_EVEN_ODD,
                                &mut m,
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
        ((self.color.red as u32) << 16) |
        ((self.color.green as u32) << 8) |
        (self.color.blue as u32)
    }
}
