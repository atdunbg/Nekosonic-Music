# Nekosonic

一款轻量的跨平台的音乐播放器，支持Windows/Linux系统，音源主要源自的网易云音乐。

## ✨ 特性

- 🔴 网易云账号登录（扫码）
- 🎵 多音质播放（标准 / 较高 / 极高 / 无损 / Hi-Res）
- 📻 私人漫游，沉浸式全屏歌词体验
- ❤️ 一键喜欢 / 取消喜欢
- 📋 歌单管理，收藏 / 取消收藏歌单
- 📅 每日推荐歌曲
- 🕐 本地播放历史记录
- 🔍 关键词搜索歌曲
- 🎤 实时滚动歌词
- 🌚 Light / Dark Mode 主题切换
- 🛠 更多特性添加中

## 📦️ 安装

访问本项目的 [Releases](https://gitea.atdunbg.xyz/atdunbg/Nekosonic-Music/releases) 页面下载安装包。


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

- [ ] MV 播放
- [ ] 音乐云盘
- [ ] 评论系统
- [ ] 下载功能
- [ ] 自定义全局快捷键
- [ ] 歌词翻译
- [ ] 更多主题

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
