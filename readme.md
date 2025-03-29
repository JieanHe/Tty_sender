## 项目简介
TTY Sender 是一个基于 Rust 开发的 Windows 应用程序，主要用于向终端窗口发送文本命令。核心功能是从本地文件读取命令模拟键盘输入发送到指定终端，主要解决一些嵌入式终端上功能不完善导致的敲命令很麻烦的问题。

### 文本发送功能：
- 单行发送， 发送鼠标光标所在行文本
- 多行批量发送， 从光标所在行开始发送，直到文件尾或者第一个空白行
- 发送到指定窗口， 在发送前需要先绑定到指定窗口
### 文件操作：
- 从文件加载命令
- 保存命令到文件

### 技术栈
编程语言: Rust
主要依赖:
winapi - Windows API 绑定
wio - COM 接口支持
lazy_static - 静态变量初始化

### 快速开始
### 构建要求
- Rust 1.60+ 工具链
- Windows 10+ 操作系统
- Windows SDK
### 安装与运行

```bash
git clone https://github.com/your-repo/tty_sender.git
cd tty_sender
cargo build --release
cargo run
```
### 使用说明
主界面：
- 左侧大文本框：编辑要发送的命令, 打开文件后会显示文件内容在此
- 底部小文本框：显示操作状态和消息
- 右侧按钮：执行各种操作
- 基本操作：
  - 点击"绑定窗口"按钮，然后拖动到目标终端窗口
  - 编辑命令文本
  - 点击"发送"按钮发送命令
- 高级功能：
  -  使用"多行发送"批量发送多行命令
  -  使用"加载"和"保存"按钮管理命令文件
### 项目结构

```plainText
tty_sender/
├── src/
│   ├── main.rs          # 程序入口
│   ├── window_data.rs   # 窗口数据结构
│   ├── input.rs         # 输入发送功能
│   ├── file_io.rs       # 文件操作
│   ├── controls.rs       # 控件管理
│   ├── utils.rs         # 工具函数
│   └── consts.rs        # 常量定义
├── Cargo.toml          # 项目配置
└── README.md           # 项目文档
```
### 贡献指南
欢迎提交 Issue 和 Pull Request。请确保：

- 代码符合 Rust 风格指南
- 新功能有对应的测试
- 重大变更更新文档
### 许可证
Apache 2.0
