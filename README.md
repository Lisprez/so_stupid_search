# so stupid search tool <阿Q的哥锐普>

# English Documentation

## install

### install from source code
1.install rust toolchain
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2.Download the file
```bash
git clone https://github.com/Lispre/so_stupid_search.git
```
3.Enter the directory contains file with name of "Cargo.toml" and Run command as blow:
```bash
cd so_stupid_search
cargo build --release
```
4.Then you will get the executable file "so_stupid_search/target/release/sss"

5.move the executable file sss to your $PATH directory
```bash
sudo mv ./target/release/sss /usr/local/bin/
```

## Usage
### search file system
```bash
  sss search-string start-directory
```
### filter command pipe
```bash
  command | sss main.go
```

## Example

### common search
```bash
sss "func main(" .
```

### search specified type of file
```bash
#only search in kubernetes yaml files
sss -t yaml fuck_str .
```
 
# 中文文档

## 安装
### 从源代码构建安装
1.安装rust编译器
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2.下载so_stupid_search源代码
```bash
git clone https://github.com/Lispre/so_stupid_search.git
```
3.进入源码根目录进行构建
```bash
cd so_stupid_search
cargo build --release
```

4.获得可执行文件 "so_stupid_search/target/release/sss"

5.将sss可执行文件复制到 $PATH 变量包含的一个目录中
```bash
sudo mv ./target/release/sss /usr/local/bin/
```
6.现在你就可以像阿Q一样使用sss了
