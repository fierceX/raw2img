#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <getopt.h>
#include <math.h>
#include "raw2jpg.h"


#define DENOISE_RATIO 400 // 降噪倍数
#define DEFAULTD_ENOISE 256 // 默认降噪参数
#define DEFAULTD_QUALITY 90 // 默认输出质量
#define MAX_BRIGHTNESS 220  // 计算平均亮度上限
#define MIN_BRIGHTNESS 20  // 计算评价亮度下限
#define MAX_BRIGHTNESS_THRESHOLD 120  // 需要进行曝光偏移的平均亮度上限
#define MIN_BRIGHTNESS_THRESHOLD 60  // 需要进行曝光偏移的平均亮度下限
#define BRIGHTNESS_THRESHOLD 110  // 确定曝光偏移的方向
#define BRIGHTNESS_THRESHOLD_FLOAT 110.0 // 计算曝光偏移的倍数

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

  // char * outputImage;
  // int width,height,colors;

  RawProcess(input,output,use_camera_wb,use_auto_wb,half_size,exp_shift,exp_shift_flag,threshold_flag,threshold,lut,lut_file,quality);

  // write_jpeg(outputImage,width,height,colors,output,quality);

  return 0;
}
