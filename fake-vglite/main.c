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
        return -1;
    }
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
    if (svg.svg == NULL) {
        printf("svg parse error\n");
        return -1;
    }
    vg_lite_buffer_t target = {
        .width = 240,
        .height = 240
    };
    printf("svglite_render: %d\n", svglite_render(&target, svg, 0, 0, 0));
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
vg_lite_matrix_t * vg_lite_get_grad_matrix(vg_lite_linear_gradient_t *grad) {return NULL;}
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
