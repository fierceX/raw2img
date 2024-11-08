# Raw 转换 Img 工具

基于 [Libraw](https://www.libraw.org/) 库进行Raw格式的解析，并且添加了一系列参数进行调整，并支持 3D Lut 滤镜。 

## 编译

### 编译Web前端

编译前端需要安装 [trunk](https://trunkrs.dev/)，安装完成后使用该工具进行编译

进入 `web` 目录，并执行编译命令：

```shell
trunk build --release
```
编译完成后会在该目录下生成 `dist` 结果目录。

### 编译raw2img
将前端的编译结果 `dist` 目录复制到 `raw2img` 目录下，并在此目录执行编译命令：

```shell
cargo build --release
```
编译完成后会在 `target/release/` 目录下生成 `raw2img` 可执行文件，该文件已经包含了前端文件。

## 使用

### 转换单个文件

使用`convert`子命令可以对单个文件进行转换，具体可以使用help命令进行参考

```shell
raw2img help convert
```

### 启动web服务

使用`server`子命令可以启动 web 服务，默认监听0.0.0.0:8081端口，其他选项参考help命令

```shell
raw2img server
```

启动后，可以通过`curl`创建用户：

```shell
curl -X POST -d "username=admin&email=admin@example.com&password=admin" http://127.0.0.1:8081/create_user
```