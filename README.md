# Skills Hub

> 管理、搜索、安装和删除 `~/.agents/skills` 目录下 Agent 技能的桌面应用。

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue)
![Vue](https://img.shields.io/badge/Vue-3-42b883)

---

## 功能

- **浏览技能** — 卡片列表展示所有已安装的 Agent 技能，支持按分类筛选和关键词搜索
- **查看文档** — 内置 Markdown 渲染器，直接阅读 `SKILL.md`
- **AI 分析** — 调用 Claude API 对技能进行深度分析，结果缓存到本地 SQLite，离线可用
- **安装技能** — 从 GitHub / GitLab / Gitee 仓库一键 Git 克隆安装（含二次确认）
- **删除技能** — 删除前展示影响说明，需二次确认
- **系统托盘** — 关闭窗口后驻留托盘，随时唤起；支持显示 / 隐藏 / 退出

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 |
| 前端 | Vue 3 + TypeScript + Vite |
| 样式 | Tailwind CSS v4（OKLCH 色彩，琥珀主题） |
| 状态 | Pinia |
| 后端 | Rust |
| 本地缓存 | SQLite（rusqlite，静态链接） |
| AI | Claude API（可配置 Base URL / 模型） |

## 目录结构

```
skills-hub/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   ├── stores/             # Pinia stores
│   ├── services/           # AI 分析 / 缓存服务
│   └── types/              # TypeScript 类型
├── src-tauri/              # Rust 后端
│   ├── src/lib.rs          # 所有 Tauri 命令
│   ├── icons/              # 应用图标（全平台）
│   └── tauri.conf.json
├── website/                # 项目官网（静态 HTML，部署到 GitHub Pages）
└── .github/workflows/      # CI：多平台构建发布 + Pages 部署
```

## 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

## 构建

```bash
# macOS — Universal Binary（同时支持 Apple Silicon 和 Intel）
npm run tauri build -- --bundles dmg,app --target universal-apple-darwin

# macOS — 仅 Apple Silicon
npm run tauri build -- --bundles dmg,app --target aarch64-apple-darwin

# macOS — 仅 Intel
npm run tauri build -- --bundles dmg,app --target x86_64-apple-darwin

# Windows（生成 NSIS 安装包）
npm run tauri build -- --bundles nsis
```

> **提示**：首次构建 Universal Binary 需要先添加目标架构：
> ```bash
> rustup target add aarch64-apple-darwin x86_64-apple-darwin
> ```

## AI 配置

首次使用 AI 分析功能，在技能详情面板的「AI 分析」标签页展开「Claude API 配置」，填入：

| 字段 | 说明 |
|---|---|
| API Key | `sk-ant-...` |
| Base URL | 留空使用官方接口；可填兼容接口地址 |
| 模型 | 默认 `claude-3-haiku-20240307` |

配置保存在 `~/Library/Application Support/com.a.skills-hub/ai_config.json`（macOS）。  
分析结果缓存在同目录下的 `ai_cache.db`，内容不变时直接读取缓存，不消耗 API 额度。

## 系统要求

- **macOS** 10.15 Catalina 及以上（Apple Silicon / Intel 均支持，Universal Binary）
- **Windows** 10 及以上（x86_64）
