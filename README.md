# Nekosonic

一款轻量的跨平台音乐播放器，支持 Windows / Linux / macOS，音源源自网易云音乐。

## ✨ 特性

### 播放

- 🎵 在线音乐播放，流式缓冲边下边播
- 🎵 多音质选择（标准 / 较高 / 极高 HQ / 无损 SQ / Hi-Res）
- 🔄 播放模式切换（列表循环 / 随机播放 / 单曲循环）
- ⏯ 播放控制（播放 / 暂停 / 上一首 / 下一首 / 进度跳转 / 音量调节）
- 📋 播放队列管理（查看队列 / 移除歌曲 / 清空队列）
- 📻 私人漫游 FM（个性化推荐，VIP 试听自动跳过）
- 🎵 本地音乐播放（支持 mp3 / flac / wav / ogg / aac / m4a / wma / opus）
- 🔊 音频输出设备选择

### 发现与浏览

- 🔍 关键词搜索歌曲 + 热门搜索标签
- 📋 歌单浏览（推荐歌单 / 排行榜 / 用户歌单 / 收藏歌单）
- 📋 歌单详情（歌曲列表 + 收藏 / 取消收藏 + 歌单评论）
- 🎤 歌手详情（热门歌曲 / 专辑 / 简介）
- 💿 专辑详情（歌曲列表 + 播放全部）
- 📅 每日推荐歌曲

### 歌词与评论

- 🎤 实时滚动歌词（自动滚动 / 点击跳转 / 渐变透明度）
- 🎤 全屏漫游模式（大封面 + 歌词 / 评论双标签页）
- 💬 歌曲评论查看（热门评论 + 无限滚动加载 + 点赞）

### 收藏与下载

- ❤️ 一键喜欢 / 取消喜欢（同步到网易云账号）
- ⬇️ 歌曲下载（带进度显示 / VIP 拦截 / 元数据保存）
- 🎵 本地音乐管理（列出 / 播放 / 删除 / 音频元数据与封面读取）
- 🕐 本地播放历史记录（最多 200 首）

### 账号

- 🔴 网易云账号登录（二维码扫码 / 手机号密码）
- 🔑 登录态持久化（重启后自动恢复）

### 系统与设置

- 📡 系统托盘（播放控制 / 显示窗口 / 退出）
- 🛡 单实例运行（防止重复启动）
- ⌨️ 自定义快捷键（应用内 + 系统全局）
- 🌚 Light / Dark Mode 主题切换
- ⚙️ 关闭窗口行为设置（每次询问 / 最小化到托盘 / 直接退出）
- 🔄 自动更新（启动静默检测 + 自定义弹窗 + 忽略版本 + 下载进度）
- 📝 更新日志查看

## 📦️ 安装

访问本项目的 [Releases](https://github.com/atdunbg/Nekosonic-Music/releases) 页面下载安装包。

## 💻 配置开发环境

```bash
# 安装前端依赖
npm install

# 启动开发服务器
npm run tauri dev

# 构建发布
npm run tauri build
```

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- Tauri CLI 2

## 🛠 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript |
| 样式 | Tailwind CSS v4 + CSS 变量主题系统 |
| 状态管理 | Pinia |
| 路由 | Vue Router 4 |
| 音频播放 | rodio (Rust) |
| 网易云 API | ncm-api-rs |
| 构建工具 | Vite 6 |

## ☑️ Todo

- [x] 评论系统
- [x] 歌曲下载
- [x] 本地音乐管理
- [x] 歌手详情页
- [x] 专辑详情页
- [x] 自定义全局快捷键
- [x] 自动更新
- [ ] MV 播放
- [ ] 音乐云盘
- [ ] 歌词翻译
- [ ] 更多主题
- [ ] 桌面歌词

欢迎提 Issue 和 Pull request。

## 📜 开源许可

本项目仅供个人学习研究使用，禁止用于商业及非法用途。

基于 [MIT license](https://opensource.org/licenses/MIT) 许可进行开源。

## 致谢

- [ncm-api-rs](https://crates.io/crates/ncm-api-rs) — 网易云音乐 API 的 Rust 封装
- [Tauri](https://tauri.app/) — 跨平台桌面应用框架
- [Vue.js](https://vuejs.org/) — 渐进式 JavaScript 框架
- [Tailwind CSS](https://tailwindcss.com/) — 实用优先的 CSS 框架
- [rodio](https://crates.io/crates/rodio) — Rust 音频播放库
