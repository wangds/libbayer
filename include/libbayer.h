#ifndef LIBBAYER_H
#define LIBBAYER_H

#include <stddef.h>

struct CRasterMut;

/*--------------------------------------------------------------*/
/* Raster                                                       */
/*--------------------------------------------------------------*/

extern struct CRasterMut *
bayerrs_raster_mut_alloc(
        size_t x, size_t y, size_t w, size_t h, size_t stride, unsigned int depth,
        unsigned char *buf, size_t buf_len);

extern void
bayerrs_raster_mut_free(struct CRasterMut *raster);

#endif
