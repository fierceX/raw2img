#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <getopt.h>
#include <math.h>
#include "libraw/libraw.h"
#include "stb_image_write.h"
// #include "jpeglib.h"

#include "lut3d.h"

void RawProcess(char * input,char * output,int use_camera_wb,int use_auto_wb,int half_size,float exp_shift,bool exp_shift_flag,bool threshold_flag,float threshold,bool lut,char *lut_file);


