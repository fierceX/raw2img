#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <getopt.h>
#include <math.h>
#include "libraw/libraw.h"
#include "jpeglib.h"

#include "lut3d.h"

#define DENOISE_RATIO 400 // 降噪倍数
#define DEFAULTD_ENOISE 256 // 默认降噪参数
#define DEFAULTD_QUALITY 90 // 默认输出质量
#define MAX_BRIGHTNESS 220  // 计算平均亮度上限
#define MIN_BRIGHTNESS 20  // 计算评价亮度下限
#define MAX_BRIGHTNESS_THRESHOLD 120  // 需要进行曝光偏移的平均亮度上限
#define MIN_BRIGHTNESS_THRESHOLD 60  // 需要进行曝光偏移的平均亮度下限
#define BRIGHTNESS_THRESHOLD 110  // 确定曝光偏移的方向
#define BRIGHTNESS_THRESHOLD_FLOAT 110.0 // 计算曝光偏移的倍数


void write_jpeg(char * img,int width, int height, int colors,const char *outout, int quality)
{
  if (colors != 1 && colors != 3)
  {
    printf("Only BW and 3-color images supported for JPEG output\n");
    return;
  }

  FILE *f = fopen(outout, "wb");
  if (!f)
    return;
  struct jpeg_compress_struct cinfo;
  struct jpeg_error_mgr jerr;
  JSAMPROW row_pointer[1]; /* pointer to JSAMPLE row[s] */
  int row_stride;          /* physical row width in image buffer */

  cinfo.err = jpeg_std_error(&jerr);
  jpeg_create_compress(&cinfo);
  jpeg_stdio_dest(&cinfo, f);
  cinfo.image_width = width; /* image width and height, in pixels */
  cinfo.image_height = height;
  cinfo.input_components = colors;                              /* # of color components per pixel */
  cinfo.in_color_space = colors == 3 ? JCS_RGB : JCS_GRAYSCALE; /* colorspace of input image */
  jpeg_set_defaults(&cinfo);
  jpeg_set_quality(&cinfo, quality, TRUE);
  jpeg_start_compress(&cinfo, TRUE);
  row_stride = width * colors; /* JSAMPLEs per row in image_buffer */
  while (cinfo.next_scanline < cinfo.image_height)
  {
    row_pointer[0] = &img[cinfo.next_scanline * row_stride];
    (void)jpeg_write_scanlines(&cinfo, row_pointer, 1);
  }
  jpeg_finish_compress(&cinfo);
  fclose(f);
  jpeg_destroy_compress(&cinfo);
}

float exposure_shift(libraw_processed_image_t *img)
{
  int mean_brightness = 0;
  int j = 0;
  for (int i = 0; i < img->data_size; i++)
  {
    if(img->data[i] < MAX_BRIGHTNESS && img->data[i] > MIN_BRIGHTNESS){
      mean_brightness += img->data[i];
      j++;
    }
    
  }

  mean_brightness = mean_brightness / j;

  float exposure_shift_value = 0.0;
  if(mean_brightness < MAX_BRIGHTNESS_THRESHOLD && mean_brightness > MIN_BRIGHTNESS_THRESHOLD){
    exposure_shift_value = (BRIGHTNESS_THRESHOLD > mean_brightness) ? pow(2.0, BRIGHTNESS_THRESHOLD_FLOAT / mean_brightness) : -pow(2.0, BRIGHTNESS_THRESHOLD_FLOAT / mean_brightness);
  }
  return exposure_shift_value;
}

#define HANDLE_FATAL_ERROR(ret)                                       \
  if (ret)                                                            \
  {                                                                   \
    fprintf(stderr, "libraw  %s\n",libraw_strerror(ret)); \
    if (LIBRAW_FATAL_ERROR(ret))                                      \
      exit(1);                                                        \
  }

#define HANDLE_ALL_ERRORS(ret)                                        \
  if (ret)                                                            \
  {                                                                   \
    fprintf(stderr, "libraw  %s\n",libraw_strerror(ret)); \
  }

#define HELP printf("RAW 转换 JPG 工具，基于 Libraw 库，支持 3d lut：\n\t -a：使用自动白平衡\n\t -w：使用相机设置的白平衡\n\t -h：输出尺寸减半\n\t -i：输入文件路径\n\t -o：输出文件路径\n\t -e：曝光偏移，值范围为 0.25-8，从降低两档到提升三档。当该值指定时，自动曝光偏移将不起作用\n\t -q：输出 JPG 质量，值范围 0-100\n\t -l：使用 lut 文件滤镜\n\t -n：降噪参数。当指定该值时，自动降噪将不起作用\n例如：raw2jpg -w -h -i input.RW2 -o output.jpg -e 2 -q 90\n");
#define VERSION printf("raw2jpg version 0.1 \nGithub: https://github.com/fierceX/raw2jpg \nGit: %s\n",GIT_SHA1);

int main(int argc, char *argv[])
{

  int use_camera_wb = 0;
  int use_auto_wb = 0;
  int half_size = 0;
  int quality = DEFAULTD_QUALITY;
  float threshold = DEFAULTD_ENOISE;
  bool threshold_flag = true;
  char input[1024];
  char output[1024];
  int io = 0;
  bool lut = false;
  char lut_file[1024];
  char expa[1024];
  float exp_shift = 0.0;
  bool exp_shift_flag = true;

  int index;
  int c;

  opterr = 0;

  while ((c = getopt(argc, argv, "vawhi:o:e:q:l:n:")) != -1)
    switch (c)
    {
    case 'v':
      VERSION;
      return 0;
    case 'a':
      use_auto_wb = 1;
      break;
    case 'w':
      use_camera_wb = 1;
      break;
    case 'h':
      half_size = 1;
      break;
    case 'i':
      snprintf(input, 1024, "%s", optarg);
      io++;
      break;
    case 'o':
      snprintf(output, 1024, "%s", optarg);
      io++;
      break;
    case 'e':
      exp_shift = atof(optarg);
      exp_shift_flag = false;
      break;
    case 'q':
      quality = atoi(optarg);
      break;
    case 'l':
      snprintf(lut_file, 1024, "%s", optarg);
      lut = true;
      break;
    case 'n':
      threshold = atof(optarg);
      threshold_flag = false;
      break;
    case '?':
      printf("无法解析参数：%c\n",optopt);
      VERSION;
      HELP;
      return 1;
    default:
      VERSION;
      HELP;
      return 1;
    }
  if(io != 2){
    printf("未指定输入输出路径\n");
    HELP;
    return 1;
  }

  int i;
  libraw_data_t *iprc = libraw_init(0);

  if (!iprc)
  {
    fprintf(stderr, "Cannot create libraw handle\n");
    exit(1);
  }

  iprc->params.half_size = 1; /* 使用一半尺寸大小计算平均亮度 */
  iprc->params.exp_correc = 1;
  iprc->params.exp_preser = 0.8;
  iprc->params.use_camera_wb = use_camera_wb;
  iprc->params.use_auto_wb = use_auto_wb;

  int ret = libraw_open_file(iprc, input);
  HANDLE_ALL_ERRORS(ret);

  printf("Processing %s (%s %s)\n", input, iprc->idata.make,
          iprc->idata.model);

  ret = libraw_unpack(iprc);
  HANDLE_ALL_ERRORS(ret);

  if(exp_shift_flag){  /* 计算平均亮度 */
    
    ret = libraw_dcraw_process(iprc);
    HANDLE_ALL_ERRORS(ret);
    
    libraw_processed_image_t *img_ = libraw_dcraw_make_mem_image(iprc, &ret);
    HANDLE_ALL_ERRORS(ret);
    float exposure_shift_value = exposure_shift(img_);
    iprc->params.exp_shift = exposure_shift_value;
  }else{
    iprc->params.exp_shift = exp_shift;
  }
  iprc->params.half_size = half_size;
  if(threshold_flag){
    threshold = threshold * (iprc->other.iso_speed / DENOISE_RATIO);
  }
  iprc->params.threshold = threshold;
  printf("曝光偏移：%f。降噪参数：%f\n",iprc->params.exp_shift,iprc->params.threshold);
  ret = libraw_dcraw_process(iprc);
  HANDLE_ALL_ERRORS(ret);
  libraw_processed_image_t *img = libraw_dcraw_make_mem_image(iprc, &ret);
  HANDLE_ALL_ERRORS(ret);

  char * outputImg = img->data;

  if(lut){
    outputImg = (unsigned char *) malloc(img->data_size * sizeof(unsigned char));
    // memcpy(outputImg, img->data, img->data_size);

    int is16bit = 0;
    //  INTERPOLATE_NEAREST
    //	INTERPOLATE_TRILINEAR
    //	INTERPOLATE_TETRAHEDRAL
    int interp_mode = INTERPOLATE_TETRAHEDRAL;
    apply_lut(lut_file, img->data, outputImg, img->width, img->height, img->width * img->colors, img->colors, interp_mode,
              is16bit);
  }
  write_jpeg(outputImg,img->width,img->height,img->colors,output,quality);
  

  libraw_close(iprc);
  return 0;
}
