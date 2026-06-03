<div align="center">

# Nekosonic

轻量跨平台桌面音乐播放器 · 网易云音乐

[![Windows](https://img.shields.io/badge/Windows-0078D4?logo=windows11&logoColor=white)](https://github.com/atdunbg/Nekosonic-Music/releases)
[![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black)](https://github.com/atdunbg/Nekosonic-Music/releases)
[![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)](https://github.com/atdunbg/Nekosonic-Music/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

[下载安装](https://github.com/atdunbg/Nekosonic-Music/releases)

---

</div>

## 🎵 播放

- 多音质选择（标准 / 较高 / HQ / SQ / Hi-Res）
- 私人漫游 FM（个性化推荐）
- 系统媒体控制集成（MPRIS / SMTC / Now Playing）
- 音频输出设备选择

## 🔍 发现

- 关键词搜索（歌曲 / 歌手 / 专辑）+ 搜索建议 + 热门搜索
- 歌单浏览（推荐 / 排行榜 / 用户 / 收藏）
- 歌手详情（热门歌曲 / 专辑 / 简介 + 关注）
- 专辑详情
- 每日推荐歌曲

## 🎤 歌词与评论

- 实时滚动歌词（ease-out 缓动 / 点击跳转 / 渐变透明度）
- 歌词翻译
- 全屏漫游模式（封面主色提取 + 歌词/评论双标签）
- 歌曲评论（无限滚动 + 点赞）

## ❤️ 收藏与下载

- 一键喜欢 / 取消喜欢（同步到网易云账号）
- 歌曲下载
- 音乐云盘（上传 / 删除 / 详情 / 存储空间 / 上传进度）
- 本地音乐（多文件夹扫描 / 封面补全）
- 下载音乐（独立管理 / 删除）

## 🎨 个性化

- 多主题色（天蓝 / 翠绿 / 玫红 / 紫罗兰 / 橙色 / 青色 / 粉色）
- 自定义快捷键（应用内 + 系统全局）
- 关闭行为设置
- 自动更新

---

## 安装

前往 [Releases](https://github.com/atdunbg/Nekosonic-Music/releases) 下载对应平台安装包。

## 配置开发环境

```bash
npm install
npm run tauri dev      # 开发
npm run tauri build    # 构建
```

> 环境要求：Node.js ≥ 18 · Rust ≥ 1.70 · Tauri CLI 2

## 技术栈

| 层级 | 技术 |
|:------|:------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Pinia |
| 样式 | Tailwind CSS v4 + CSS 变量主题 |
| 音频解码 | symphonia + ringbuf |
| 媒体控制 | souvlaki |
| 网易云 API | ncm-api-rs |
| 构建 | Vite 6 |

## Todo

- [x] 评论查看
- [x] 歌曲下载
- [x] 本地音乐
- [x] 歌手详情页
- [x] 专辑详情页
- [x] 自定义全局快捷键
- [x] 自动更新
- [x] 歌词翻译
- [x] 更多主题
- [x] 音乐云盘
- [ ] MV 播放
- [ ] 桌面歌词

欢迎提 Issue 和 Pull request。

## 开源许可

本项目仅供个人学习研究使用，禁止用于商业及非法用途。基于 [MIT License](https://opensource.org/licenses/MIT) 开源。
