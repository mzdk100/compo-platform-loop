# Compo Platform Loop

[English Documentation](README.md)

---

为 [Compo](https://github.com/mzdk100/compo) 声明式响应式组件框架提供的跨平台事件循环实现。该库为 Windows、macOS、iOS 和 Android 提供平台特定的事件循环集成，使 Compo 应用程序能够在不同操作系统上原生运行。

## 特性

- **跨平台支持**：为 Windows、macOS、iOS 和 Android 提供原生事件循环集成
- **Compo 集成**：与 Compo 组件框架无缝集成
- **平台特定优化**：在每个平台上使用原生 API 以获得最佳性能
- **异步运行时**：基于 Compo 的单线程异步运行时构建，提供高性能
- **零依赖**：最小的外部依赖，充分利用平台原生 API

## 平台支持

| 平台 | 状态 | 事件循环实现 |
|------|------|-------------|
| Windows  | ✅ | Win32 消息循环 |
| macOS    | ✅ | NSRunLoop 配合 NSTimer |
| iOS      | ✅ | NSRunLoop 配合 NSTimer |
| Android  | ✅ | JNI 配合 Java MainLoop |

## 快速开始

### 安装

在你的 `Cargo.toml` 中添加：

```shell
cargo add compo compo-platform-loop
```

### 基本用法

```rust
use compo::prelude::*;
use compo_platform_loop::prelude::run;
use tracing::info;

#[component]
async fn hello() {
    println!("Hello, world!");
    info!("Hello, world!");
}

#[cfg(not(target_os = "android"))]
fn main() {
    run(hello);
}

#[cfg(target_os = "android")]
fn main(vm: jni::JavaVM) {
    run(vm, hello);
}
```

## 示例

仓库包含不同平台的示例：

### 桌面端 (Windows/macOS/Linux)

```bash
cd examples/desktop
cargo run
```

### Android

```bash
cd examples/android
# 详细设置说明请参见 examples/android/README.md
./run.sh  # Windows 上使用 run.bat
```

### iOS

```bash
cd examples/ios
# 详细设置说明请参见 examples/ios/README.md
# 在 Xcode 中打开 CompoPlatformLoopExample.xcodeproj 并运行
```

## 平台特定详情

### Windows
- 使用 Win32 `PeekMessageW` 进行非阻塞消息轮询
- 与 Windows 消息泵集成以实现原生窗口处理
- 支持 `WM_QUIT` 消息以实现优雅关闭

### macOS
- 使用 `NSRunLoop` 配合 `NSTimer` 进行定期运行时轮询
- 与 `NSApplication` 集成以实现原生应用生命周期
- 在主线程上运行，轮询间隔为 0.01 秒

### iOS
- 类似于 macOS，但不使用 `NSApplication`
- 使用 `NSRunLoop` 配合 `NSTimer` 进行运行时轮询
- 专为 iOS 应用生命周期集成而设计

### Android
- 使用 JNI 在 Rust 和 Java 之间建立桥梁
- Java `MainLoop` 类处理 Android 事件循环
- 注册原生方法用于运行时轮询

## API 参考

### `run` 函数

启动平台特定事件循环的主要入口点：

```rust
// 大多数平台
pub fn run<'a, C, F>(entry: F)
where
    C: Component<'a> + 'a,
    F: AsyncFn(Weak<C>) + 'a,

// Android 平台
pub fn run<C, F>(vm: JavaVM, entry: F)
where
    C: Component<'static> + 'static,
    F: AsyncFn(Weak<C>) + 'static,
```

## 不同平台的构建

### 桌面端
```bash
cargo build --release
```

### Android
需要 Android NDK 和适当的工具链设置：
```bash
cargo apk2 build --release
```

### iOS
需要 Xcode 和 iOS 工具链：
```bash
cargo build --release --target aarch64-apple-ios
```

## 贡献

欢迎贡献！请随时提交问题和拉取请求。

## 许可证

Apache-2.0

## 相关项目

- [Compo](https://github.com/mzdk100/compo) - 核心声明式响应式组件框架
- [cargo-apk2](https://github.com/mzdk100/cargo-apk2) - 使用Cargo轻松构建安卓应用的命令行工具