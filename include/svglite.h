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

#ifndef _svglite_h_
#define _svglite_h_

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include "vg_lite.h"

typedef void* svglite_svg_t;
typedef void* svglite_fontdb_t;

const char* svglite_version(void);
svglite_fontdb_t svglite_fontdb_create(void);
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
