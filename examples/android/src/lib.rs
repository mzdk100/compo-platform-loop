#![cfg(target_os = "android")]

use {
    jni::{
        JavaVM,
        sys::{JNI_VERSION_1_6, jint},
    },
    std::ffi::c_void,
    tracing::Level,
    tracing_logcat::{LogcatMakeWriter, LogcatTag},
    tracing_subscriber::{fmt, fmt::format::Format},
};

include!("../../hello.rs");

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    let tag = LogcatTag::Fixed(env!("CARGO_APK2_PACKAGE").to_owned());
    let writer = LogcatMakeWriter::new(tag).expect("Failed to initialize logcat writer");
    fmt()
        .event_format(Format::default().with_level(false).without_time())
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .init();

    main(vm);

    // 返回支持的 JNI 版本
    JNI_VERSION_1_6
}
