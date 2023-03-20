#ifndef _svglite_h_
#define _svglite_h_

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "vg_lite.h"

typedef struct svglite_svg {
    void* svg;
} svglite_svg_t;

void svglite_free(svglite_svg_t svg);
svglite_svg_t svglite_svg_from_data(const unsigned char* data, size_t len);
vg_lite_error_t svglite_render(vg_lite_buffer_t* target, svglite_svg_t svg, vg_lite_fill_t fill_rule, vg_lite_blend_t blend, vg_lite_quality_t quality);

#ifdef __cplusplus
}
#endif

#endif
