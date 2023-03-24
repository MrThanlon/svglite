#ifndef _svglite_h_
#define _svglite_h_

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "vg_lite.h"

typedef void* svglite_svg_t;
typedef void* svglite_fontdb_t;

svglite_fontdb_t svglite_fontdb_create();
void svglite_fontdb_free(svglite_fontdb_t db);
void svglite_fontdb_load_font_data(svglite_fontdb_t db, const unsigned char* data, size_t len);
void svglite_fontdb_load_fonts_dir(svglite_fontdb_t db, const char* dir);
void svglite_fontdb_load_system_fonts(svglite_fontdb_t db);
size_t svglite_fontdb_len(svglite_fontdb_t db);

void svglite_free(svglite_svg_t svg);
svglite_svg_t svglite_svg_from_data(const unsigned char* data, size_t len);
vg_lite_error_t svglite_render(vg_lite_buffer_t* target,
                               svglite_svg_t svg,
                               vg_lite_fill_t fill_rule,
                               vg_lite_blend_t blend,
                               vg_lite_quality_t quality,
                               const svglite_fontdb_t db);

#ifdef __cplusplus
}
#endif

#endif
