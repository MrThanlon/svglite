#include <stddef.h>
#include <stdint.h>
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
vg_lite_buffer_t* fb = NULL;
svglite_fontdb_t fonts = NULL;

void clean(void) {
    if (fonts != NULL)
        svglite_fontdb_free(fonts);
    if (svg_buffer != NULL)
        free(svg_buffer);
    if (f != NULL)
        fclose(f);
    if (fb != NULL) {
        vg_lite_free(fb);
        vg_lite_close();
    }
}

// 320x240 VG_LITE_BGRA8888 phy address
void display_vo_layer0 (uint32_t addr) {
    const uint32_t base = 0x90840288;
    const uint32_t end = 0x90840294;
    // devmem 0x90840000 32 0x12345678
    char command[40];
    for (uint32_t reg = base; reg <= end; reg += 4) {
        sprintf(command, "devmem %08X 32 %08X", reg, addr);
        system(command);
    }
}

int main(int argc, char* argv[]) {
    vg_lite_error_t error;

    if (argc < 5) {
        printf("Usage: %s <width> <height> <input SVG> <output raw> [fonts dir]...\n", argv[0]);
        return -1;
    }
    if (argc > 5) {
        fonts = svglite_fontdb_create();
        unsigned fonts_count = 0;
        // load font
        for (unsigned i = 5; i < argc; i++) {
            svglite_fontdb_load_fonts_dir(fonts, argv[i]);
            printf("load %lu fonts from %s\n", svglite_fontdb_len(fonts) - fonts_count, argv[i]);
            fonts_count = svglite_fontdb_len(fonts);
        }
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
    if (svg == NULL) {
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
    buffer.width = width;
    buffer.height = height;
    buffer.format = VG_LITE_BGRA8888;
    error = vg_lite_allocate(&buffer);
    if (error != VG_LITE_SUCCESS) {
        printf("vg_lite_allocate() error: %d\n", error);
        return -1;
    }
    fb = &buffer;
    error = vg_lite_clear(fb, NULL, 0xffffffff);
    if (error != VG_LITE_SUCCESS) {
        printf("vg_lite_clear() error: %d", error);
        return -1;
    }
    error = svglite_render(fb, svg, VG_LITE_FILL_NON_ZERO, VG_LITE_BLEND_NONE, VG_LITE_HIGH, fonts);
    if (error != VG_LITE_SUCCESS) {
        printf("svglite_render() error: %d", error);
        return -1;
    }
    // FIXME: magic number
    if (fb->width == 320 && fb->height == 240) {
        system("devmem 0x90840000 ");
    }
    f = fopen(argv[4], "w");
    if (f == NULL) {
        perror("fopen() error");
        return -1;
    }
    s = fwrite(fb->memory, 1, fb->stride * fb->height, f);
    if (s < fb->stride * fb->height) {
        perror("fwrite() error");
        return -1;
    }
    fclose(f);
    f = NULL;
    printf("written %lu bytes to %s done\n", s, argv[4]);
    return 0;
}
