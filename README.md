# RAW格式转换成JPG小工具

该工具主要依赖 [Libraw](https://www.libraw.org/) 库，由于该库并没有合适的可执行文件，所以包装了一个小工具。  
在此基础上，集成了由 [cpuimage 整理自 FFmpeg 中的3D lut处理算法](https://github.com/cpuimage/FFmpeg_Lut3D)。

## 编译
需要安装libraw库和jpeg库，然后在此目录下运行：
```
make
```
即可完成编译，生成可执行文件`raw2jpg`。

## 使用
```
RAW 转换 JPG 工具，基于 Libraw 库：
         -a：使用自动白平衡
         -w：使用相机设置的白平衡
         -h：输出尺寸减半
         -i：输入文件路径
         -o：输出文件路径
         -e：曝光偏移，值范围为 0.25-8，从降低两档到提升三档。当该值指定时，自动曝光偏移将不起作用
         -q：输出 JPG 质量，值范围 0-100
         -l：使用 lut 文件滤镜
         -n：降噪参数。当指定该值时，自动降噪将不起作用
例如：raw2jpg -w -h -i input.RW2 -o output.jpg -e 2 -q 90
```