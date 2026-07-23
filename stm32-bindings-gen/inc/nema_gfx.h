#ifndef NEMA_GFX_BINDINGS_H_
#define NEMA_GFX_BINDINGS_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* Core 2.5D raster API */
#include "nema_core.h"
#include "nema_raster.h"
#include "nema_error.h"
#include "nema_provisional.h"

/* Vector graphics (NeoChromVG on supported parts) */
#include "nema_vg.h"
#include "nema_vg_font.h"
#include "nema_vg_tsvg.h"

/* Fonts, transitions, easing helpers */
#include "nema_font.h"
#include "nema_transitions.h"
#include "nema_easing.h"

/* Platform heap used by NemaGFX command lists */
#include "tsi_malloc.h"

#ifdef __cplusplus
}
#endif

#endif /* NEMA_GFX_BINDINGS_H_ */
