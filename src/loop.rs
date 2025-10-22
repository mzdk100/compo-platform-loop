//! Platform-specific event loop implementations for the Compo framework.
//!
//! This module provides cross-platform event loop integration that allows Compo
//! applications to run natively on different operating systems. Each platform
//! uses its native event loop mechanism:
//!
//! - **Windows**: Win32 message loop with PeekMessage for non-blocking processing
//! - **macOS**: NSApplication with NSRunLoop and NSTimer for periodic polling
//! - **iOS**: NSRunLoop with NSTimer for periodic polling (without NSApplication)
//! - **Android**: JNI integration with Java MainLoop for Android event system
//!
//! The module exports platform-appropriate `run` functions that initialize the
//! Compo runtime and integrate it with the platform's native event loop.

use compo::prelude::*;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};
#[cfg(target_os = "ios")]
use {
    block2::RcBlock,
    objc2::{ClassType, msg_send},
    objc2_foundation::{NSRunLoop, NSString, NSTimer},
};
#[cfg(target_os = "android")]
use {
    jni::{AttachGuard, JNIEnv, JavaVM, NativeMethod, errors::Result as JniResult},
    std::{
        any::Any,
        cell::{Cell, RefCell},
        ptr::null_mut,
    },
    tracing::error,
};

#[cfg(target_os = "macos")]
use {
    block2::RcBlock,
    objc2::{ClassType, msg_send},
    objc2_foundation::{NSRunLoop, NSString, NSTimer},
};

// Thread-local storage for Android runtime and component management.
//
// On Android, we use thread-local storage to maintain the Compo runtime and
// component instances. This is necessary because Android's JNI callbacks
// need access to the runtime from the same thread where it was created.
#[cfg(target_os = "android")]
thread_local! {
    /// The Compo runtime instance for processing async tasks
    static RT: Rc<Runtime<'static, ()>> = Rc::new(Runtime::new());
    /// Storage for the root component instance
    static COMPONENT: Cell<Rc<dyn Any>> = Cell::new(Rc::new(()));
    /// Storage for the JavaVM instance to allow JNI calls from any thread
    ///
    /// This is used to store the JavaVM instance obtained during JNI_OnLoad
    /// so that we can attach threads to the JVM when needed for callbacks.
    /// The RefCell allows mutable access to the JavaVM instance when attaching
    /// new threads or performing JNI operations.
    static JAVA_VM: RefCell<JniResult<JavaVM>> = RefCell::new(unsafe { JavaVM::from_raw(null_mut()) });
}

/// Handles Windows message processing for the event loop.
///
/// This function processes Windows messages using PeekMessage instead of GetMessage
/// to avoid blocking the event loop. It handles WM_QUIT messages for graceful shutdown
/// and dispatches other messages to their appropriate window procedures.
///
/// # Arguments
/// * `r#loop` - Reference to the Loop instance for controlling the event loop
#[cfg(windows)]
fn handle_windows_message(r#loop: &Loop) {
    // Use PeekMessage instead of GetMessage because GetMessage blocks until a message is available
    unsafe {
        let mut msg = MSG::default();
        // Check if there are messages in the queue without blocking
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            // If it's a WM_QUIT message, exit the loop
            if msg.message == WM_QUIT {
                r#loop.quit();
                break;
            }

            // Translate virtual key messages
            let _ = TranslateMessage(&msg);
            // Dispatch message to window procedure
            DispatchMessageW(&msg);
        }
    }
}

/// Runs the platform-specific event loop with the given entry component.
///
/// This function initializes and starts the appropriate event loop for the current platform:
/// - **Windows**: Uses Win32 message loop with PeekMessage for non-blocking message processing
/// - **macOS**: Uses NSApplication with NSRunLoop and NSTimer for periodic runtime polling
/// - **iOS**: Uses NSRunLoop with NSTimer for periodic runtime polling (without NSApplication)
///
/// The function creates a Compo runtime, spawns the entry component as an async task,
/// and integrates with the platform's native event loop to ensure proper execution
/// of async components.
///
/// # Type Parameters
/// * `C` - The component type that implements `Component<'a>`
/// * `F` - The async function type that takes a `Weak<C>` and returns a future
///
/// # Arguments
/// * `entry` - The entry point async function that will be executed as the root component
///
/// # Platform-specific behavior
/// - **Windows**: Registers a message handler and runs the Loop with Windows message processing
/// - **macOS**: Creates NSApplication, sets up a timer for runtime polling, and runs the app loop
/// - **iOS**: Sets up NSRunLoop with a timer for runtime polling (suitable for iOS apps)
///
/// # Examples
/// ```rust
/// //! Platform-specific event loop implementations for the Compo framework.
/// //!
/// //! This module provides cross-platform event loop integration that allows Compo
/// //! applications to run natively on different operating systems. Each platform
/// //! uses its native event loop mechanism:
/// //!
/// //! - **Windows**: Win32 message loop with PeekMessage for non-blocking processing
/// //! - **macOS**: NSApplication with NSRunLoop and NSTimer for periodic polling
/// //! - **iOS**: NSRunLoop with NSTimer for periodic polling (without NSApplication)
/// //! - **Android**: JNI integration with Java MainLoop for Android event system
/// //!
/// //! The module exports platform-appropriate `run` functions that initialize the
/// //! Compo runtime and integrate it with the platform's native event loop.
///
/// use compo::prelude::*;
/// use compo_platform_loop::prelude::run;
///
/// #[component]
/// async fn app() {
///     println!("Hello from Compo!");
/// }
///
/// fn main() {
///     run(app);
/// }
/// ```
#[cfg(not(target_os = "android"))]
pub fn run<'a, C, F>(entry: F)
where
    C: Component<'a> + 'a,
    F: AsyncFn(Weak<C>) + 'a,
{
    #[cfg(windows)]
    Loop::new()
        .register_poll_handler(handle_windows_message)
        .run(entry);

    #[cfg(target_os = "ios")]
    {
        // 创建运行时和组件
        let rt = Rc::new(Runtime::new());
        let rt_weak = Rc::downgrade(&rt);
        let c = Rc::new(C::new(rt_weak.clone()));
        let c_weak = Rc::downgrade(&c);

        // 启动异步任务
        rt.spawn(async move { entry(c_weak).await });

        // 获取主线程的运行循环
        let run_loop = NSRunLoop::mainRunLoop();

        // 创建一个定时器，用于定期轮询 Runtime
        let poll_block = RcBlock::new(move || {
            // 轮询 Runtime 以推进异步任务
            if let Some(rt) = rt_weak.upgrade() {
                rt.poll_all();
            }
        });

        // 创建一个重复的定时器，每 0.01 秒轮询一次
        let timer: *mut NSTimer = unsafe {
            msg_send![NSTimer::class(),
                scheduledTimerWithTimeInterval: 0.01,
                repeats: true,
                block: &*poll_block
            ]
        };

        // 将定时器添加到运行循环中
        let mode = NSString::from_str("NSDefaultRunLoopMode");
        let _: () = unsafe { msg_send![&run_loop, addTimer: timer, forMode: &*mode] };

        // 运行主循环，这会阻塞当前线程
        #[cfg(not(feature = "application"))]
        run_loop.run();
        #[cfg(feature = "application")]
        if let Some(mtm) = objc2::MainThreadMarker::new() {
            objc2_ui_kit::UIApplication::main(None, None, mtm)
        } else {
            tracing::error!("Can't run on non-main-thread.")
        }
    }

    #[cfg(target_os = "macos")]
    {
        // 创建运行时和组件
        let rt = Rc::new(Runtime::new());
        let rt_weak = Rc::downgrade(&rt);
        let c = Rc::new(C::new(rt_weak.clone()));
        let c_weak = Rc::downgrade(&c);

        // 启动异步任务
        rt.spawn(async move { entry(c_weak).await });

        // 获取主线程的运行循环
        let run_loop = NSRunLoop::mainRunLoop();

        // 创建一个定时器，用于定期轮询 Runtime
        let poll_block = RcBlock::new(move || {
            // 轮询 Runtime 以推进异步任务
            if let Some(rt) = rt_weak.upgrade() {
                rt.poll_all();
            }
        });

        // 创建一个重复的定时器，每 0.01 秒轮询一次
        let timer: *mut NSTimer = unsafe {
            msg_send![NSTimer::class(),
                scheduledTimerWithTimeInterval: 0.01,
                repeats: true,
                block: &*poll_block
            ]
        };

        // 将定时器添加到运行循环中
        // 创建 NSDefaultRunLoopMode 字符串
        let mode = NSString::from_str("NSDefaultRunLoopMode");
        let _: () = unsafe { msg_send![&run_loop, addTimer: timer, forMode: &*mode] };

        #[cfg(not(feature = "application"))]
        run_loop.run();
        #[cfg(feature = "application")]
        if let Some(mtm) = objc2::MainThreadMarker::new() {
            let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
            app.activate();

            // 运行应用程序主循环，这会阻塞当前线程
            app.run();
        } else {
            tracing::error!("Can't run on non-main-thread.")
        }
    }
}

/// Runs the Android-specific event loop with JNI integration.
///
/// This function sets up the event loop for Android applications using JNI (Java Native Interface).
/// It creates a Compo runtime in thread-local storage, spawns the entry component, and integrates
/// with the Android Java MainLoop class for proper event loop execution.
///
/// The function registers a native method `poll_all` that can be called from Java to advance
/// the async runtime, enabling proper integration with Android's event system.
///
/// # Type Parameters
/// * `C` - The component type that implements `Component<'static>` (must be 'static for Android)
/// * `F` - The async function type that takes a `Weak<C>` and returns a future (must be 'static)
///
/// # Arguments
/// * `vm` - The JavaVM instance provided by the Android runtime
/// * `entry` - The entry point async function that will be executed as the root component
///
/// # Android Integration
/// This function:
/// 1. Creates a thread-local Compo runtime and component
/// 2. Spawns the entry component as an async task
/// 3. Calls the Java `MainLoop.run()` method to start the Android event loop
/// 4. Registers the native `poll_all` method for runtime advancement
///
/// # Examples
/// ```rust
/// //! Platform-specific event loop implementations for the Compo framework.
/// //!
/// //! This module provides cross-platform event loop integration that allows Compo
/// //! applications to run natively on different operating systems. Each platform
/// //! uses its native event loop mechanism:
/// //!
/// //! - **Windows**: Win32 message loop with PeekMessage for non-blocking processing
/// //! - **macOS**: NSApplication with NSRunLoop and NSTimer for periodic polling
/// //! - **iOS**: NSRunLoop with NSTimer for periodic polling (without NSApplication)
/// //! - **Android**: JNI integration with Java MainLoop for Android event system
/// //!
/// //! The module exports platform-appropriate `run` functions that initialize the
/// //! Compo runtime and integrate it with the platform's native event loop.
///
/// use compo::prelude::*;
/// use compo_platform_loop::prelude::run;
/// use jni::JavaVM;
///
/// #[component]
/// async fn app() {
///     println!("Hello from Android!");
/// }
///
/// fn main(vm: JavaVM) {
///     run(vm, app);
/// }
/// ```
#[cfg(target_os = "android")]
pub fn run<C, F>(vm: JavaVM, entry: F)
where
    C: Component<'static> + 'static,
    F: AsyncFn(Weak<C>) + 'static,
{
    JAVA_VM.set(Ok(vm));
    RT.with(|rt| {
        let rt_weak = Rc::downgrade(rt);
        let c = Rc::new(C::new(rt_weak.clone()));
        let c_weak = Rc::downgrade(&c);

        // 启动异步任务
        rt.spawn(async move { entry(c_weak).await });
        COMPONENT.set(c);
    });
    vm_exec(|mut env| {
        const CLASS: &str = "rust/compo/MainLoop";
        if let Err(e) = env.call_static_method(CLASS, "run", "()V", &[]) {
            error!(?e, "Run failed.");
        }
        let method = NativeMethod {
            name: "poll_all".into(),
            sig: "()V".into(),
            fn_ptr: poll_all as *mut _,
        };
        if let Err(e) = env.register_native_methods(CLASS, &[method]) {
            error!(?e, "Register native method failed.");
        }
    });
}

/// Native method called from Java to advance the Compo runtime.
///
/// This function is registered as a JNI native method and called by the Android
/// Java MainLoop to poll and advance all pending async tasks in the Compo runtime.
/// It accesses the thread-local runtime and calls `poll_all()` to process any
/// ready futures.
///
/// # Safety
/// This function is marked as `unsafe` because it's a C-style callback function
/// that will be called from Java via JNI. The JNI environment parameter is
/// currently unused but required by the JNI interface.
///
/// # Arguments
/// * `_env` - The JNI environment (unused in current implementation)
#[cfg(target_os = "android")]
unsafe extern "C" fn poll_all(_env: JNIEnv) {
    RT.with(|rt| rt.poll_all());
}

/// Executes a closure with an attached JNI environment.
///
/// This function provides thread-safe access to the Java VM environment by:
/// 1. Borrowing the stored JavaVM instance
/// 2. Attaching the current thread to the JVM
/// 3. Executing the provided closure with the attached environment
///
/// # Type Parameters
/// * `F` - Closure type that takes an `AttachGuard` (must be thread-safe)
///
/// # Safety
/// The closure must not perform any operations that could cause thread-local
/// data corruption or violate JNI safety rules.
///
/// # Error Handling
/// Logs errors if:
/// - JavaVM is not initialized (must call `run` first)
/// - Thread attachment fails
#[cfg(target_os = "android")]
pub fn vm_exec<F>(f: F)
where
    F: for<'a> FnOnce(AttachGuard<'a>),
{
    JAVA_VM.with_borrow_mut(move |vm| match vm {
        Ok(vm) => match vm.attach_current_thread() {
            Ok(env) => f(env),
            Err(e) => error!(?e, "Can't attach current thread."),
        },
        Err(e) => error!(?e, "Java VM is not initialized, please call the `run` function and set the correct JavaVM first."),
    })
}
