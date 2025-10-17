use {compo::prelude::*, compo_platform_loop::prelude::run,tracing::info};

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
