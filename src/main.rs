#![allow(dead_code)]
// mod vglite;
mod vglite_util;

use vglite_util::*;
use std::{env::args, fs::{read_to_string, self}, mem::transmute, ffi::c_void, io::Write};
use usvg::{
    self,
    Node,
    NodeKind::{Group, Path, Image, Text},
    Paint::{Color, LinearGradient, RadialGradient, Pattern},
    Transform, Visibility,
    PathSegment::{*, self}
};

struct VGLiteConfig {
    target: *mut vg_lite_buffer,
    fill_rule: vg_lite_fill_t,
    /// Not used
    blend: vg_lite_blend_t,
    quality: vg_lite_quality_t,
    format: vg_lite_format_t
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
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
        handle: unsafe { transmute([0;2]) },
        memory: unsafe { transmute([0;2]) },
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
    }

    // svg
    let svg = read_to_string(args[3].as_str()).expect("error: file not found");
    let svg = usvg::Tree::from_str(svg.as_str(), &usvg::Options::default()).expect("error: parse SVG failed");

    // dfs
    let mut dummy = unsafe { transmute::<[u32;64],vg_lite_buffer>([0;64]) };
    dfs(&svg.root, &mut Transform::default(), &VGLiteConfig {
        target: &mut dummy,
        fill_rule: 0,
        blend: 0,
        quality: 0,
        format: 0
    });
    error = unsafe { vg_lite_finish() };
    if error != vg_lite_error_VG_LITE_SUCCESS {
        println!("error: vg_lite_finish() failed with code {}", error);
    }
    let data = unsafe {
        std::slice::from_raw_parts(buffer.memory as *const u8, (buffer.stride * buffer.height) as usize)
    };
    let mut output = fs::File::create(args[4].as_str()).expect("error: create output file");
    output.write(data).expect("error: write");
    unsafe {
        vg_lite_free(&mut buffer);
        vg_lite_close();
    };
}

fn dfs(node: &Node, mat: &Transform, config: &VGLiteConfig) -> u32 {
    match node.borrow().to_owned() {
        Group(group) => {
            // let mut mat = mat.clone();
            // mat.transform(&group.transform);
            for child in node.children() {
                dfs(&child, &group.transform, config);
            }
        },
        Path(path) => {
            if path.visibility != Visibility::Visible {
                return vg_lite_error_VG_LITE_SUCCESS;
            }
            if let Some(fill) = path.fill {
                // build path
                let mut path_data = Vec::new();
                let v: Vec<PathSegment>= path.data.segments().collect();
                dbg!(v);
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
                let mut p = vg_lite_path {
                    bounding_box: [0., 0., unsafe { *config.target }.width as f32, unsafe { *config.target }.height as f32],
                    quality: config.quality,
                    format: config.format,
                    uploaded: unsafe { transmute::<[u32;8], vg_lite_hw_memory>([0;8]) },
                    path_length: (path_data.len() * 4) as i32,
                    path: path_data.as_mut_ptr() as *mut c_void,
                    path_changed: 1,
                    pdata_internal: 0
                };
                match fill.paint {
                    Color(color) => {
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
                            &mut vg_lite_matrix {
                                m: [[mat.a as f32, mat.c as f32, mat.e as f32],
                                    [mat.b as f32, mat.d as f32, mat.f as f32],
                                    [0., 0., 1.]
                                ]
                            },
                            config.blend,
                            c
                        ) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            return error;
                        }
                    },
                    LinearGradient(_lg) => {
                    },
                    RadialGradient(_rg) => {
                    },
                    Pattern(_p) => {
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

trait Transformer {
    fn transform(&mut self, t: &Transform) -> &mut Self;
}

impl Transformer for Transform {
    /// t * self
    fn transform(&mut self, t: &Transform) -> &mut Self {
        self.a = self.a * t.a + self.c * t.b;
        self.b = self.b * t.a + self.d * t.b;
        self.c = self.a * t.c + self.c * t.d;
        self.d = self.b * t.c + self.d * t.d;
        self.e = self.a * t.e + self.c * t.f + self.e;
        self.f = self.b * t.e + self.d * t.f + self.f;
        self
    }
}

fn multiply(t1: &Transform, t2: &Transform) -> Transform {
    Transform {
        a: t1.a * t2.a + t1.c * t2.b,
        b: t1.b * t2.a + t1.d * t2.b,
        c: t1.a * t2.c + t1.c * t2.d,
        d: t1.b * t2.c + t1.d * t2.d,
        e: t1.a * t2.e + t1.c * t2.f + t1.e,
        f: t1.b * t2.e + t1.d * t2.f + t1.f
    }
}
