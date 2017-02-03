#ifndef LIBBAYER_H
#define LIBBAYER_H

#include <stddef.h>

struct CRasterMut;

enum {
    BAYERRS_SUCCESS = 0,
    BAYERRS_ERROR = 1,
    BAYERRS_WRONG_RESOLUTION = 2,
    BAYERRS_WRONG_DEPTH = 3,
};

#define CFA_BGGR 0
#define CFA_GBRG 1
#define CFA_GRBG 2
#define CFA_RGGB 3

/*--------------------------------------------------------------*/
/* Demosaicing algorithms                                       */
/*--------------------------------------------------------------*/

extern unsigned int
bayerrs_demosaic_none(
        const unsigned char *src, size_t src_len,
        unsigned int depth, unsigned int big_endian, unsigned int cfa,
        struct CRasterMut *dst);

extern unsigned int
bayerrs_demosaic_nearest_neighbour(
        const unsigned char *src, size_t src_len,
        unsigned int depth, unsigned int big_endian, unsigned int cfa,
        struct CRasterMut *dst);

extern unsigned int
bayerrs_demosaic_linear(
        const unsigned char *src, size_t src_len,
        unsigned int depth, unsigned int big_endian, unsigned int cfa,
        struct CRasterMut *dst);

extern unsigned int
bayerrs_demosaic_cubic(
        const unsigned char *src, size_t src_len,
        unsigned int depth, unsigned int big_endian, unsigned int cfa,
        struct CRasterMut *dst);

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
