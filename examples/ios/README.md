# iOS Build

1. Set your development team ID using environment variable:
   ```bash
   export DEVELOPMENT_TEAM=YOUR_TEAM_ID
   ```
   Or set it in Xcode Build Settings under User-Defined settings;
2. Use Xcode to open the CompoPlatformLoopExample.xcodeproj project;
3. Click the `run` button in the toolbar;
4. If you encounter errors about being unable to link System libraries during build, please run the following command first:
   ```bash
   cargo build --release --target aarch64-apple-ios
   ```
   Then re-execute step 3 to run.
   This is because Xcode may have set some special environment variables (not yet identified) that cause conflicts.