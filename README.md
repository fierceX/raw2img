# Raw 转换 Img 工具

基于 [Libraw](https://www.libraw.org/) 库进行Raw格式的解析，并且添加了一系列参数进行调整，并支持 3D Lut 滤镜。 

## 编译 Raw2Img 工具

进入 `raw2img` 目录，执行编译命令：
```shell
cargo build --release
```
编译完成后会在该目录的 `target/release/` 目录下生成 `raw2img` 可执行文件，使用方法如下：
```shell
Usage: raw2img [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>          输入文件路径
  -o, --output <OUTPUT>        输出文件路径
  -l, --lut <LUT>              使用 lut 文件滤镜 [default: ]
  -a, --auto-wb                使用自动白平衡或相机白平衡
  -h, --half-size              输出尺寸减半
  -e, --exp-shift <EXP_SHIFT>  曝光偏移，值范围为 0.25-8，从降低两档到提升三档。当该值指定时，自动曝光偏移将不起作用 [default: -3]
  -q, --quality <QUALITY>      输出质量，值范围 0-100 [default: 90]
  -n, --noise <NOISE>          降噪参数。当指定该值时，自动降噪将不起作用 [default: -1]
  -h, --help                   Print help
  -V, --version                Print version
```
## 编译和使用Web项目

### 编译Web前端

编译前端需要安装 [trunk](https://trunkrs.dev/)，安装完成后使用该工具进行编译

进入 `web` 目录，并执行编译命令：
```shell
trunk build --release
```
编译完成后会在该目录下生成 `dist` 结果目录。

### 编译Web后端
将前端的编译结果 `dist` 目录复制到 `server` 目录下，并在此目录执行编译命令：
```shell
cargo build --release
```
编译完成后会在 `target/release/` 目录下生成 `server` 可执行文件，该文件已经包含了前端文件。

### 使用

直接执行 `server` 会启动 web 服务，并监听 8081 端口，需要提前在同文件夹下新建 lut 文件夹，该文件夹存放以 `cube` 后缀的 lut 文件。  

启动后，会依据 Cookie 在同文件夹下的 `tmp` 目录创建用户目录，上传 RAW 格式文件会存放在此，点击图像上面的名称，会进入编辑页面，在该页面可以自定义参数进行 RAW 格式的转换。