#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <assert.h>

int apply_lut(char *filename, const uint8_t *indata, uint8_t *outdata, int width, int height, int linesize, int depth,
          int interpolation, int is16bit);

enum interp_mode {
    INTERPOLATE_NEAREST,
    INTERPOLATE_TRILINEAR,
    INTERPOLATE_TETRAHEDRAL,
    NB_INTERP_MODE
};