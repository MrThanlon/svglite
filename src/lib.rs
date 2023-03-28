#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("./vg_lite.rs");

use std::{
    slice,
    io::{Read, Result},
    mem::transmute,
    ptr::{null_mut},
    ffi::{c_void, CStr},
    f64::consts::PI,
    os::raw::c_char
};
use png::{BitDepth, ColorType};
use usvg::{
    self,
    Node,
    NodeKind::{Group, Path, Image, Text},
    Paint::{Color, LinearGradient, RadialGradient, Pattern},
    Transform, Visibility,
    PathSegment::*, NodeExt, Stop, Units, Tree,
    ImageKind::*,
};
use usvg_text_layout::*;

extern crate jpeg_decoder as jpeg;

struct VGLiteConfig {
    target: *mut vg_lite_buffer,
    fill_rule: vg_lite_fill_t,
    /// Not used
    blend: vg_lite_blend_t,
    quality: vg_lite_quality_t
}

struct VecReader<'a> {
    vec: &'a Vec<u8>,
    position: usize,
}

impl<'a> VecReader<'a> {
    fn new(vec: &'a Vec<u8>) -> VecReader<'a> {
        VecReader {
            vec,
            position: 0,
        }
    }
}

impl<'a> Read for VecReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let available_bytes = self.vec.len() - self.position;

        if available_bytes == 0 {
            Ok(0)
        } else {
            let bytes_to_read = buf.len().min(available_bytes);
            let end = self.position + bytes_to_read;
            buf[..bytes_to_read].copy_from_slice(&self.vec[self.position..end]);
            self.position = end;
            Ok(bytes_to_read)
        }
    }
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
extern "C" fn svglite_fontdb_create() -> *mut fontdb::Database {
    Box::into_raw(Box::new(fontdb::Database::new()))
}

#[no_mangle]
extern "C" fn svglite_fontdb_free(db: *mut fontdb::Database) {
    unsafe { Box::from_raw(db) };
}

#[no_mangle]
extern "C" fn svglite_fontdb_load_font_data(db: *mut fontdb::Database, data: *const u8, len: usize) {
    let db = unsafe {&mut*db};
    let data = unsafe {
        slice::from_raw_parts(data, len)
    };
    db.load_font_data(Vec::from(data));
}

#[no_mangle]
extern "C" fn svglite_fontdb_load_fonts_dir(db: *mut fontdb::Database, dir: *const c_char) {
    let db = unsafe {&mut*db};
    let dir = unsafe {
        CStr::from_ptr(dir)
    }.to_str().unwrap();
    db.load_fonts_dir(dir);
}

#[no_mangle]
extern "C" fn svglite_fontdb_load_system_fonts(db: *mut fontdb::Database) {
    let db = unsafe {&mut*db};
    db.load_system_fonts();
}

#[no_mangle]
extern "C" fn svglite_fontdb_len(db: *mut fontdb::Database) -> usize {
    let db = unsafe {&mut*db};
    db.len()
}

#[no_mangle]
extern "C" fn svglite_render(
    target: &mut vg_lite_buffer,
    svg: *mut Tree,
    fill_rule: vg_lite_fill_t,
    blend: vg_lite_blend_t,
    quality: vg_lite_quality_t,
    db: *mut fontdb::Database
) -> vg_lite_error {
    let svg = unsafe {&*svg};
    let db = if db.is_null() {
        None
    } else {
        Some(unsafe {&*db})
    };
    let mut viewbox_mat = Transform::default();
    viewbox_mat.scale(target.width as f64 / svg.view_box.rect.width(), target.height as f64 / svg.view_box.rect.height());
    viewbox_mat.translate(-svg.view_box.rect.left(), -svg.view_box.rect.top());
    let error = dfs(&svg.root, &viewbox_mat, &VGLiteConfig {
        target,
        fill_rule,
        blend,
        quality,
    }, db);
    if error != vg_lite_error_VG_LITE_SUCCESS {
        // println!("error: dfs() failed with code {}", error);
        return error;
    }

    unsafe { vg_lite_finish() }
}

fn dfs(node: &Node, mat: &Transform, config: &VGLiteConfig, db: Option<&fontdb::Database>) -> u32 {
    let mut m = mat.clone();
    m.append(&node.transform());
    match node.borrow().to_owned() {
        Group(_group) => {
            for child in node.children() {
                let e = dfs(&child, &m, config, db);
                if e != vg_lite_error_VG_LITE_SUCCESS {
                    return e
                }
            }
            vg_lite_error_VG_LITE_SUCCESS
        },
        Path(path) => {
            if path.visibility != Visibility::Visible || path.data.is_empty() {
                return vg_lite_error_VG_LITE_SUCCESS;
            }
            if let Some(fill) = path.fill {
                // build path
                let mut path_data = Vec::new();
                for seg in path.data.segments() {
                    match seg {
                        MoveTo { x, y } => {
                            path_data.push(unsafe { transmute::<u32, f32>(VLC_OP_MOVE) });
                            path_data.push(x as f32);
                            path_data.push(y as f32);
                        },
                        LineTo { x, y } => {
                            path_data.push(unsafe { transmute::<u32, f32>(VLC_OP_LINE) });
                            path_data.push(x as f32);
                            path_data.push(y as f32);
                        },
                        CurveTo { x1, y1, x2, y2, x, y } => {
                            path_data.push(unsafe { transmute::<u32, f32>(VLC_OP_CUBIC) });
                            path_data.push(x1 as f32);
                            path_data.push(y1 as f32);
                            path_data.push(x2 as f32);
                            path_data.push(y2 as f32);
                            path_data.push(x as f32);
                            path_data.push(y as f32);
                        },
                        ClosePath => {
                            path_data.push(unsafe { transmute::<u32, f32>(VLC_OP_CLOSE) });
                        }
                    }
                }
                path_data.push(unsafe { transmute::<u32, f32>(VLC_OP_END) });

                let bbox = if let Some(bbox) = node.calculate_bbox() {
                    bbox
                } else {
                    eprintln!("Warning: path can't read bounding box, ID: {}", path.id);
                    return vg_lite_error_VG_LITE_SUCCESS;
                };

                let mut mr = vg_lite_matrix::from_transform(&m);
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

                match fill.paint {
                    Color(color) => {
                        let c = ((fill.opacity.to_u8() as u32) << 24) |
                        ((color.red as u32) << 16) |
                        ((color.green as u32) << 8) |
                        ((color.blue as u32) << 0);

                        let error = unsafe { vg_lite_draw(
                            config.target,
                            &mut p,
                            config.fill_rule,
                            &mut mr,
                            config.blend,
                            c
                        ) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            eprintln!("Error at {}:{}", file!(), line!());
                            return error;
                        }
                    },
                    LinearGradient(lg) => {
                        if lg.base.stops.len() > 16 {
                            println!("Error: linearGradient stops must not bigger than 16");
                            return vg_lite_error_VG_LITE_NOT_SUPPORT;
                        }

                        // build gradient
                        let mut grad: vg_lite_linear_gradient = unsafe { transmute([0usize;53]) };
                        let mut error = unsafe { vg_lite_init_grad(&mut grad) };
                        if error != vg_lite_error_VG_LITE_SUCCESS {
                            eprintln!("Error at {}:{}", file!(), line!());
                            return error;
                        }

                        let mut colors: Vec<vg_lite_color_t> = (&lg.base.stops).into_iter().map(|x| {
                            x.get_u32()
                        }).collect();

                        // stop is 0 to 255, use matrix to scale
                        let mut stops: Vec<u32> = (&lg.base.stops).into_iter().map(|x| {
                            x.offset.to_u8() as u32
                        }).collect();

                        let mat;
                        unsafe {
                            vg_lite_set_grad(
                                &mut grad,
                                stops.len() as u32,
                                colors.as_mut_ptr(),
                                stops.as_mut_ptr()
                            );
                            vg_lite_update_grad(&mut grad);
                            mat = &mut *vg_lite_get_grad_matrix(&mut grad);
                        }
                        // gradient transform
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

                        mat.update_transform(&grad_mat);
                        unsafe {
                            error = vg_lite_draw_gradient(
                                config.target,
                                &mut p,
                                vg_lite_fill_VG_LITE_FILL_EVEN_ODD,
                                &mut mr,
                                &mut grad,
                                config.blend
                            );
                            if error != vg_lite_error_VG_LITE_SUCCESS {
                                eprintln!("Error {}:{}", file!(), line!());
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
            vg_lite_error_VG_LITE_SUCCESS
        },
        Image(image) => {
            if image.visibility != Visibility::Visible {
                return vg_lite_error_VG_LITE_SUCCESS;
            }
            m.translate(image.view_box.rect.x(), image.view_box.rect.y());
            // allocate new buffer to do BLITs
            let mut buffer;
            let error = match image.kind {
                SVG(tree) => {
                    buffer = vg_lite_buffer::default(
                        image.view_box.rect.width() as i32,
                        image.view_box.rect.height() as i32,
                        vg_lite_buffer_format_VG_LITE_RGBA8888
                    );
                    let error = unsafe {vg_lite_allocate(&mut buffer)};
                    if error != vg_lite_error_VG_LITE_SUCCESS {
                        return error;
                    }
                    dfs(&tree.root, &Transform::default(), &VGLiteConfig {
                        target: &mut buffer,
                        fill_rule: config.fill_rule,
                        blend: config.blend,
                        quality: config.quality
                    }, db)
                },
                JPEG(jpeg) => {
                    let jpeg = jpeg.as_ref();
                    let jpeg_reader = VecReader::new(jpeg);
                    let mut decoder = jpeg::Decoder::new(jpeg_reader);
                    let image_info;
                    if let Err(_) = decoder.read_info() {
                        eprintln!("read info error at {}:{}", file!(), line!());
                        return vg_lite_error_VG_LITE_NOT_SUPPORT;
                    }
                    if let Some(info) = decoder.info() {
                        image_info = info;
                        buffer = vg_lite_buffer::default(
                            image_info.width as i32,
                            image_info.height as i32,
                            match info.pixel_format {
                                jpeg::PixelFormat::L8 => vg_lite_buffer_format_VG_LITE_L8,
                                jpeg::PixelFormat::L16 => vg_lite_buffer_format_VG_LITE_L8,
                                jpeg::PixelFormat::CMYK32 => vg_lite_buffer_format_VG_LITE_RGBA8888,
                                jpeg::PixelFormat::RGB24 => vg_lite_buffer_format_VG_LITE_BGRA8888,
                            }
                        );
                    } else {
                        eprintln!("read info error at {}:{}", file!(), line!());
                        return vg_lite_error_VG_LITE_NOT_SUPPORT;
                    }
                    let error = unsafe {vg_lite_allocate(&mut buffer)};
                    if error != vg_lite_error_VG_LITE_SUCCESS {
                        return error;
                    }
                    let buffer_memory = unsafe {
                        slice::from_raw_parts_mut(buffer.memory as *mut u8, (buffer.height * buffer.stride) as usize)
                    };
                    if let Ok(image) = decoder.decode() {
                        match image_info.pixel_format {
                            jpeg::PixelFormat::L8 => {
                                // memcpy
                                buffer_memory.copy_from_slice(&image);
                            },
                            jpeg::PixelFormat::L16 => {
                                // TODO
                                buffer_memory.fill(0);
                            },
                            jpeg::PixelFormat::RGB24 => {
                                // RGB24 to RGB32
                                if image.len() * 4 != (buffer.height * buffer.stride) as usize * 3 {
                                    return vg_lite_error_VG_LITE_INVALID_ARGUMENT;
                                }
                                convert_rgb24_to_rgb32(&image, buffer_memory);
                            },
                            jpeg::PixelFormat::CMYK32 => {
                                // TODO
                                buffer_memory.fill(0);
                            }
                        }
                    } else {
                        eprintln!("image decode error at {}:{}", file!(), line!());
                        return vg_lite_error_VG_LITE_NOT_SUPPORT;
                    }
                    m.scale(
                        image.view_box.rect.width() / image_info.width as f64,
                        image.view_box.rect.height() / image_info.height as f64
                    );
                    vg_lite_error_VG_LITE_SUCCESS
                }
                PNG(png) => {
                    let decoder = png::Decoder::new(VecReader::new(png.as_ref()));
                    if let Ok(mut reader) = decoder.read_info() {
                        let info = reader.info();
                        if info.bit_depth == BitDepth::Sixteen {
                            eprintln!("image bit depth 16 error at {}:{}", file!(), line!());
                            return vg_lite_error_VG_LITE_NOT_SUPPORT;
                        }
                        m.scale(
                            image.view_box.rect.width() / info.width as f64,
                            image.view_box.rect.width() / info.height as f64
                        );
                        match info.color_type {
                            ColorType::Grayscale => {
                                if reader.output_buffer_size() > (info.width * info.height) as usize {
                                    // imposible
                                    eprintln!("image size error at {}:{}", file!(), line!());
                                    return vg_lite_error_VG_LITE_NOT_SUPPORT;
                                }
                                // copy
                                buffer = vg_lite_buffer::default(
                                    info.width as i32,
                                    info.height as i32,
                                    vg_lite_buffer_format_VG_LITE_L8
                                );
                                let error = unsafe {vg_lite_allocate(&mut buffer)};
                                if error != vg_lite_error_VG_LITE_SUCCESS {
                                    return error;
                                }
                                let buffer_memory = unsafe {
                                    slice::from_raw_parts_mut(buffer.memory as *mut u8, (buffer.height * buffer.stride) as usize)
                                };
                                if let Err(_) = reader.next_frame(buffer_memory) {
                                    unsafe {vg_lite_free(&mut buffer)};
                                    eprintln!("image decode error at {}:{}", file!(), line!());
                                    return vg_lite_error_VG_LITE_NOT_SUPPORT;
                                }
                            },
                            ColorType::Indexed => {
                                // TODO: CLUT
                                eprintln!("image format error at {}:{}", file!(), line!());
                                return vg_lite_error_VG_LITE_NOT_SUPPORT;
                            },
                            ColorType::GrayscaleAlpha => {
                                // TODO
                                eprintln!("image format error at {}:{}", file!(), line!());
                                return vg_lite_error_VG_LITE_NOT_SUPPORT;
                            },
                            ColorType::Rgb => {
                                let mut rgb24_buffer = vec![0;reader.output_buffer_size()];
                                if let Ok(output_info) = reader.next_frame(&mut rgb24_buffer) {
                                    // convert rgb24 to rgb32
                                    buffer = vg_lite_buffer::default(
                                        output_info.width as i32,
                                        output_info.height as i32,
                                        vg_lite_buffer_format_VG_LITE_RGBA8888
                                    );
                                    let error = unsafe {vg_lite_allocate(&mut buffer)};
                                    if error != vg_lite_error_VG_LITE_SUCCESS {
                                        return error;
                                    }
                                    let buffer_memory = unsafe {
                                        slice::from_raw_parts_mut(
                                            buffer.memory as *mut u8,
                                            (buffer.height * buffer.stride) as usize
                                        )
                                    };
                                    convert_rgb24_to_rgb32(&rgb24_buffer, buffer_memory);
                                } else {
                                    eprintln!("read output info error at {}:{}", file!(), line!());
                                    return vg_lite_error_VG_LITE_NOT_SUPPORT;
                                }
                            },
                            ColorType::Rgba => {
                                if reader.output_buffer_size() > (info.width * info.height * 4) as usize {
                                    // imposible
                                    eprintln!("image size error at {}:{}", file!(), line!());
                                    return vg_lite_error_VG_LITE_NOT_SUPPORT;
                                }
                                // copy
                                buffer = vg_lite_buffer::default(
                                    info.width as i32,
                                    info.height as i32,
                                    vg_lite_buffer_format_VG_LITE_RGBA8888
                                );
                                let error = unsafe {vg_lite_allocate(&mut buffer)};
                                if error != vg_lite_error_VG_LITE_SUCCESS {
                                    return error;
                                }
                                let buffer_memory = unsafe {
                                    slice::from_raw_parts_mut(buffer.memory as *mut u8, (buffer.height * buffer.stride) as usize)
                                };
                                if let Err(_) = reader.next_frame(buffer_memory) {
                                    unsafe {vg_lite_free(&mut buffer)};
                                    eprintln!("image allocate at {}:{}", file!(), line!());
                                    return vg_lite_error_VG_LITE_NOT_SUPPORT;
                                }
                            }
                        }
                    } else {
                        eprintln!("image read info error at {}:{}", file!(), line!());
                        return vg_lite_error_VG_LITE_NOT_SUPPORT;
                    }
                    vg_lite_error_VG_LITE_SUCCESS
                },
                GIF(_) => { return vg_lite_error_VG_LITE_NOT_SUPPORT;}
            };
            if error != vg_lite_error_VG_LITE_SUCCESS {
                return error;
            }
            // BLITs
            let error = unsafe {
                vg_lite_blit(
                    config.target,
                    &mut buffer,
                    &mut vg_lite_matrix::from_transform(&m),
                    vg_lite_blend_VG_LITE_BLEND_NONE,
                    0,
                    vg_lite_filter_VG_LITE_FILTER_BI_LINEAR
                )
            };
            if error != vg_lite_error_VG_LITE_SUCCESS {
                return error;
            }
            let error = unsafe {
                vg_lite_finish()
            };
            if error != vg_lite_error_VG_LITE_SUCCESS {
                return error;
            }
            unsafe {
                vg_lite_free(&mut buffer)
            }
        },
        Text(text) => {
            if let Some(db) = db {
                if let Some(paths) = text.convert(db, m) {
                    // dfs_dbg(&paths);
                    dfs(&paths, &Transform::default(), config, None)
                } else {
                    eprintln!("Error: <text> rendering error at {}:{}", file!(), line!());
                    vg_lite_error_VG_LITE_NOT_SUPPORT
                }
            } else {
                eprintln!("Warning: no suitable font, ignore this <text> element");
                vg_lite_error_VG_LITE_SUCCESS
            }
        }
    }
}

#[allow(unused)]
fn dfs_dbg(n: &Node) {
    match n.borrow().to_owned() {
        Group(_) => {
            for n in n.children() {
                dfs_dbg(&n);
            }
        },
        Path(p) => {
            dbg!(p);
        },
        _ => {}
    }
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

impl vg_lite_buffer {
    fn default(width: i32, height: i32, format: vg_lite_buffer_format) -> vg_lite_buffer {
        vg_lite_buffer {
            width, height, format,
            stride: 0, tiled: 0,
            handle: null_mut(),
            memory: null_mut(),
            address: 0,
            yuv: vg_lite_yuvinfo {
                swizzle: 0,
                yuv2rgb: 0,
                uv_planar: 0,
                v_planar: 0,
                alpha_planar: 0,
                uv_stride: 0,
                v_stride: 0,
                alpha_stride: 0,
                uv_height: 0,
                v_height: 0,
                uv_memory: null_mut(),
                v_memory: null_mut(),
                uv_handle: null_mut(),
                v_handle: null_mut()
            },
            image_mode: 0,transparency_mode: 0,fc_enable: 0,fc_buffer: [vg_lite_fc_buffer {
                width: 0, height: 0, stride: 0, handle: null_mut(), memory: null_mut(), address: 0,color:0
            };3]
        }
    }
}

impl vg_lite_matrix {
    fn from_transform(t: &Transform) -> vg_lite_matrix {
        vg_lite_matrix {
            m: [[t.a as f32, t.c as f32, t.e as f32],
                [t.b as f32, t.d as f32, t.f as f32],
                [0., 0., 1.]
            ]
        }
    }

    fn update_transform(&mut self, t: &Transform) {
        self.m[0][0] = t.a as f32;
        self.m[0][1] = t.c as f32;
        self.m[0][2] = t.e as f32;
        self.m[1][0] = t.b as f32;
        self.m[1][1] = t.d as f32;
        self.m[1][2] = t.f as f32;
        self.m[2][0] = 0.;
        self.m[2][1] = 0.;
        self.m[2][2] = 1.;
    }
}

pub fn convert_rgb24_to_rgb32(src: &[u8], dst: &mut [u8]) {
    let chunks = src.chunks_exact(3);
    let mut dst_chunks = dst.chunks_exact_mut(4);

    for chunk in chunks {
        dst_chunks
            .next()
            .unwrap()
            .copy_from_slice(&[chunk[0], chunk[1], chunk[2], 0xFF]);
    }
}
