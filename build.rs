use {
    encoding_rs::GBK,
    std::{
        env::var,
        path::Path,
        process::{Command, Stdio, exit},
    },
};

fn main() {
    // 检查目标平台是否为 Android
    let target = var("TARGET").unwrap_or_default();
    if target.contains("android")
        && let Ok(classes_dir) = var("CARGO_APK2_CLASSES_DIR")
        && let Ok(java_home) = var("CARGO_APK2_JAVA_HOME")
        && let Ok(android_jar) = var("CARGO_APK2_ANDROID_JAR")
    {
        let java_home = Path::new(&java_home);
        let output = Command::new(java_home.join("bin").join("javac"))
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-d")
            .arg(&classes_dir)
            .arg("-classpath")
            .arg(&android_jar)
            .arg("src/MainLoop.java")
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();

        if !output.status.success() {
            GBK.decode(&output.stderr).0.lines().for_each(|line| {
                eprintln!("{}", line);
            });
            exit(output.status.code().unwrap_or_default());
        } else if !output.stdout.is_empty() {
            GBK.decode(&output.stdout).0.lines().for_each(|line| {
                eprintln!("{}", line);
            });
        }

        println!("cargo:rerun-if-changed=src/MainLoop.java");
        println!("cargo:rerun-if-env-changed=CARGO_APK2_ARTIFACT");
    }

    // 确保在目标平台变化时重新运行构建脚本
    println!("cargo:rerun-if-env-changed=TARGET");
}
