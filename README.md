# Compo Platform Loop

[中文文档](README-zh-CN.md)

---

A cross-platform event loop implementation for the [Compo](https://github.com/mzdk100/compo) declarative and reactive component framework. This library provides platform-specific event loop integration for Windows, macOS, iOS, and Android, enabling Compo applications to run natively on different operating systems.

## Features

- **Cross-Platform Support**: Native event loop integration for Windows, macOS, iOS, and Android
- **Compo Integration**: Seamlessly integrates with the Compo component framework
- **Platform-Specific Optimizations**: Uses native APIs for optimal performance on each platform
- **Async Runtime**: Built on Compo's single-threaded async runtime for high performance
- **Zero Dependencies**: Minimal external dependencies, leveraging platform-native APIs

## Platform Support

| Platform | Status | Event Loop Implementation |
|----------|--------|---------------------------|
| Windows  | ✅ | Win32 Message Loop |
| macOS    | ✅ | NSRunLoop with NSTimer |
| iOS      | ✅ | NSRunLoop with NSTimer |
| Android  | ✅ | JNI with Java MainLoop |

## Quick Start

### Installation

Add this to your `Cargo.toml`:

```shell
cargo add compo compo-platform-loop
```

### Basic Usage

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

## Examples

The repository includes examples for different platforms:

### Desktop (Windows/macOS/Linux)

```bash
cd examples/desktop
cargo run
```

### Android

```bash
cd examples/android
# See examples/android/README.md for detailed setup instructions
./run.sh  # or run.bat on Windows
```

### iOS

```bash
cd examples/ios
# See examples/ios/README.md for detailed setup instructions
# Open CompoPlatformLoopExample.xcodeproj in Xcode and run
```

## Platform-Specific Details

### Windows
- Uses Win32 `PeekMessageW` for non-blocking message polling
- Integrates with Windows message pump for native window handling
- Supports `WM_QUIT` message for graceful shutdown

### macOS
- Uses `NSRunLoop` with `NSTimer` for periodic runtime polling
- Integrates with `NSApplication` for native app lifecycle
- Runs on the main thread with 0.01s polling interval

### iOS
- Similar to macOS but without `NSApplication`
- Uses `NSRunLoop` with `NSTimer` for runtime polling
- Designed for iOS app lifecycle integration

### Android
- Uses JNI to bridge between Rust and Java
- Java `MainLoop` class handles the Android event loop
- Native methods registered for runtime polling

## API Reference

### `run` Function

The main entry point for starting the platform-specific event loop:

```rust
// For most platforms
pub fn run<'a, C, F>(entry: F)
where
    C: Component<'a> + 'a,
    F: AsyncFn(Weak<C>) + 'a,

// For Android
pub fn run<C, F>(vm: JavaVM, entry: F)
where
    C: Component<'static> + 'static,
    F: AsyncFn(Weak<C>) + 'static,
```

## Building for Different Platforms

### Desktop
```bash
cargo build --release
```

### Android
Requires Android NDK and appropriate toolchain setup:
```bash
cargo apk2 build --release
```

### iOS
Requires Xcode and iOS toolchain:
```bash
cargo build --release --target aarch64-apple-ios
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

Apache-2.0

## Related Projects

- [Compo](https://github.com/mzdk100/compo) - The core declarative and reactive component framework
- [cargo-apk2](https://github.com/mzdk100/cargo-apk2) - A command-line tool for easily building Android applications with Cargo