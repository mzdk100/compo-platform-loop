package rust.compo;

import android.os.Handler;
import android.os.Looper;
import android.util.Log;

/**
 * MainLoop 类用于处理 Rust 代码与 Android 主线程之间的交互
 * 这个类被 Rust 代码通过 JNI 调用
 */
public class MainLoop {
    private static final String TAG = "MainLoop";
    private static final Handler mainHandler = new Handler(Looper.getMainLooper());
    private static final int POLL_INTERVAL_MS = 16; // 约 60fps
    private static boolean isRunning = false;
    
    /**
     * 启动主循环，定期调用 Rust 的 poll_all 方法
     * 此方法由 Rust 代码调用
     */
    public static void run() {
        if (isRunning) return;
        Log.i(TAG, "Starting MainLoop");
        isRunning = true;
        
        // 在主线程上定期执行 poll_all
        mainHandler.post(new Runnable() {
            @Override
            public void run() {
                if (isRunning) {
                    // 调用 Rust 的 native 方法
                    poll_all();
                    // 继续循环
                    mainHandler.postDelayed(this, POLL_INTERVAL_MS);
                }
            }
        });
    }
    
    /**
     * 停止主循环
     */
    public static void stop() {
        Log.i(TAG, "Stopping MainLoop");
        isRunning = false;
    }
    
    /**
     * 由 Rust 代码实现的 native 方法，用于轮询 Rust 运行时
     */
    private static native void poll_all();
}