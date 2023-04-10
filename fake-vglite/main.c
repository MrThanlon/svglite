/* Copyright (c) 2022, Canaan Bright Sight Co., Ltd
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 * notice, this list of conditions and the following disclaimer in the
 * documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND
 * CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
 * INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING,
 * BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
 * WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
 * NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <vg_lite.h>
#include <svglite.h>

ssize_t read_all(const char* path, unsigned char** buffer) {
    size_t file_size = 0;
    struct stat file_stat;
    if (stat(path, &file_stat) < 0) {
        *buffer = NULL;
        return -1;
    }
    file_size = file_stat.st_size;
    FILE* f = fopen(path, "r");
    if (f == NULL) {
        *buffer = NULL;
        return -1;
    }
    *buffer = malloc(file_size);
    if (fread(*buffer, 1, file_size, f) < file_size) {
        free(*buffer);
        *buffer = NULL;
        fclose(f);
        return -1;
    }
    fclose(f);
    return file_size;
}

int main(int argc, char* argv[]) {
    if (argc != 2) {
        printf("Usage: %s <SVG>\n", argv[0]);
        return -1;
    }
    unsigned char* buffer = NULL;
    ssize_t file_size = -1;
    if ((file_size = read_all(argv[1], &buffer)) < 0) {
        perror("read error");
        return -1;
    }
    svglite_svg_t svg = svglite_svg_from_data(buffer, file_size);
    if (svg == NULL) {
        printf("svg parse error\n");
        return -1;
    }
    vg_lite_buffer_t target = {
        .width = 240,
        .height = 240
    };
    svglite_fontdb_t db = svglite_fontdb_create();
    svglite_fontdb_load_fonts_dir(db, "/mnt/c/Windows/Fonts");
    printf("db: %p, len: %lu\n", db, svglite_fontdb_len(db));
    printf("svglite_render: %d\n", svglite_render(&target, svg, 0, 0, 0, db));
    return 0;
}

vg_lite_error_t vg_lite_draw(vg_lite_buffer_t *target,
                                 vg_lite_path_t   *path,
                                 vg_lite_fill_t    fill_rule,
                                 vg_lite_matrix_t *matrix,
                                 vg_lite_blend_t   blend,
                                 vg_lite_color_t   color) {return 0;}
vg_lite_error_t vg_lite_update_grad(vg_lite_linear_gradient_t *grad) {return 0;}
vg_lite_error_t vg_lite_finish(void) {return 0;}
vg_lite_error_t vg_lite_init_grad(vg_lite_linear_gradient_t *grad) {return 0;}
vg_lite_error_t vg_lite_set_grad(vg_lite_linear_gradient_t *grad,
                                     uint32_t count,
                                     uint32_t *colors,
                                     uint32_t *stops) {return 0;}
vg_lite_matrix_t * vg_lite_get_grad_matrix(vg_lite_linear_gradient_t *grad) {return &grad->matrix;}
vg_lite_error_t vg_lite_clear(vg_lite_buffer_t *target,
                                  vg_lite_rectangle_t *rectangle,
                                  vg_lite_color_t color) {return 0;}
vg_lite_error_t vg_lite_draw_gradient(vg_lite_buffer_t *target,
                                          vg_lite_path_t *path,
                                          vg_lite_fill_t fill_rule,
                                          vg_lite_matrix_t *matrix,
                                          vg_lite_linear_gradient_t *grad,
                                          vg_lite_blend_t blend) {return 0;}
vg_lite_error_t vg_lite_free(vg_lite_buffer_t *buffer) {return 0;}
vg_lite_error_t vg_lite_allocate(vg_lite_buffer_t *buffer) {
    switch (buffer->format) {
        case VG_LITE_RGBA8888: buffer->stride = buffer->width * 4; break;
        case VG_LITE_INDEX_8:
        case VG_LITE_L8: buffer->stride = buffer->width; break;
        default: return VG_LITE_NOT_SUPPORT;
    }
    buffer->memory = malloc(buffer->height * buffer->stride);
    if (buffer->memory != NULL) {
        return VG_LITE_SUCCESS;
    } else {
        return VG_LITE_OUT_OF_MEMORY;
    }
}
vg_lite_error_t vg_lite_blit(vg_lite_buffer_t *target, vg_lite_buffer_t *source, vg_lite_matrix_t *matrix, vg_lite_blend_t blend, vg_lite_color_t color, vg_lite_filter_t filter) {
    static unsigned i = 0;
    char name[20];
    printf("blit %d:%dx%d@%u\n", i, source->width, source->height, source->format);
    sprintf(name, "out-%d.bin", i);
    FILE* f = fopen(name, "w");
    if (f == NULL) {
        perror("open file error");
        return VG_LITE_GENERIC_IO;
    }
    if (fwrite(source->memory, 1, source->stride * source->height, f) < source->stride * source->height) {
        perror("write file error");
        fclose(f);
        return VG_LITE_GENERIC_IO;
    }
    fclose(f);
    i += 1;
    return VG_LITE_SUCCESS;
}
