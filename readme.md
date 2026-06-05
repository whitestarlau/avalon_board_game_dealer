# 阿瓦隆发牌工具

这是一个阿瓦隆局域网发牌工具，通过部署在局域网服务器上，玩家的手机通过访问指定网址，实现自动发牌。

之所以实现这一个服务，是因为项目作者和朋友玩阿瓦隆，拿了一晚上的邪恶方牌。希望通过算法伪随机的方式来避免这种现象。

## 使用

1. 确保已安装 Rust 和 Node.js
2. 构建前端：
   ```bash
   cd frontend && npm install && npm run build
   ```
3. 启动服务端：
   ```bash
   cd backend && cargo run
   ```
4. 打开 http://127.0.0.1:3004

所有玩家在首页选择编号 → 点"准备" → 等待全员就绪 → 查看自己的角色。

管理后台：http://127.0.0.1:3004/admin（可查看本局所有人角色及历史记录）

## 角色配置（7人局）

- **正义方：** 梅林、派西维尔、忠臣 ×2
- **邪恶方：** 莫甘娜、刺客、奥伯伦

## 技术框架

后端 Rust + Axum 框架，前端 Vue 3 + Vite。单服务部署，前端构建产物由 Axum 托管。

## 本地开发

```bash
# 前端热更新
cd frontend && npm run dev

# 后端（单独终端）
cd backend && cargo run
```

## 部署

将 `backend/` 目录部署到服务器，确保 `backend/static/` 目录存在（由 `cd frontend && npm run build` 生成），运行 `cargo run` 即可。

## Android 部署

将服务器运行在 Android 手机上：

```bash
# 1. 安装 Android NDK 和 Rust Android 目标
rustup target add aarch64-linux-android
cargo install cargo-ndk

# 2. 构建
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/<版本号>
./build_android.sh

# 3. APK 生成在 android/app/build/outputs/apk/debug/app-debug.apk
cd android && ./gradlew assembleDebug
```

安装 APK 到手机，打开热点，点击"启动服务器"，朋友扫码连接即可。
