#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include "svglite.h"
#include "vg_lite.h"

unsigned char* svg_buffer = NULL;
FILE* f = NULL;
vg_lite_buffer_t buffer;
vg_lite_buffer_t* fb;

void clean(void) {
    if (svg_buffer != NULL)
        free(svg_buffer);
    if (f != NULL)
        fclose(f);
    if (fb != NULL) {
        vg_lite_free(fb);
        vg_lite_close();
    }
}

int main(int argc, char* argv[]) {
    vg_lite_error_t error;

    if (argc != 5) {
        printf("Usage: %s <width> <height> <input SVG> <output raw>\n", argv[0]);
        return -1;
    }
    int width = atoi(argv[1]);
    int height = atoi(argv[2]);
    if (width == 0 || height == 0) {
        printf("width and height must be greater than 0\n");
        return -1;
    }
    struct stat svg_stat;
    if (stat(argv[3], &svg_stat) < 0) {
        perror("stat failed");
        return -1;
    }
    size_t file_size = svg_stat.st_size;
    svg_buffer = malloc(file_size);
    if (svg_buffer == NULL) {
        perror("malloc() error");
        return -1;
    }
    atexit(clean);

    f = fopen(argv[3], "r");
    if (f == NULL) {
        perror("fopen() error");
        return -1;
    }

    ssize_t s = fread(svg_buffer, svg_stat.st_size, 1, f);
    if (s == 0 || s < svg_stat.st_size) {
        if (feof(f)) {
            file_size = s;
        }
        if (ferror(f)) {
            perror("fread() error");
            return -1;
        }
    }
    fclose(f);
    f = NULL;

    svglite_svg_t svg = svglite_svg_from_data(svg_buffer, file_size);
    if (svg.svg == NULL) {
        printf("svglite_svg_from_data() error\n");
        return -1;
    }
    free(svg_buffer);
    svg_buffer = NULL;
    error = vg_lite_init(width, height);
    if (error != VG_LITE_SUCCESS) {
        printf("vg_lite_init() error: %d\n", error);
        return -1;
    }
    vg_lite_buffer_t buffer = {.width=width,.height=height,.format=VG_LITE_RGBA8888};
    error = vg_lite_allocate(&buffer);
    if (error != VG_LITE_SUCCESS) {
        printf("vg_lite_allocate() error: %d\n", error);
        return -1;
    }
    error = svglite_render(&buffer, svg, VG_LITE_FILL_NON_ZERO, VG_LITE_BLEND_NONE, VG_LITE_HIGH);
    if (error != VG_LITE_SUCCESS) {
        printf("svglite_render() error: %d", error);
        return -1;
    }
    f = fopen(argv[4], "w");
    if (f == NULL) {
        perror("fopen() error");
        return -1;
    }
    s = fwrite(buffer.memory, 1, buffer.stride * buffer.height, f);
    if (s < buffer.stride * buffer.height) {
        perror("fwrite() error");
        return -1;
    }
    return 0;
}
