use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};

/// SQLite 连接状态，用 Mutex 包裹保证线程安全
struct DbState(Mutex<Connection>);

/// 在应用数据目录初始化 SQLite 数据库，创建 ai_cache 表
fn init_db(app: &tauri::AppHandle) -> Result<Connection, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let db_path = data_dir.join("ai_cache.db");
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS ai_cache (
            dir_name     TEXT    PRIMARY KEY,
            content_hash TEXT    NOT NULL,
            data         TEXT    NOT NULL,
            timestamp    INTEGER NOT NULL
        )
    ",
    )
    .map_err(|e| e.to_string())?;
    Ok(conn)
}

/// Skill 元数据结构，对应 SKILL.md frontmatter
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub dir_name: String,
    pub applies_to: Vec<String>,
    pub depends_on: Vec<String>,
    pub has_skill_file: bool,
}

/// 安装结果
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub dir_name: String,
    pub message: String,
}

/// 获取 ~/.agents/skills 目录路径（跨平台）
fn get_skills_dir() -> PathBuf {
    // macOS/Linux 使用 HOME；Windows 优先 USERPROFILE，其次 HOMEDRIVE+HOMEPATH
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .or_else(|_| {
            // Windows fallback: HOMEDRIVE + HOMEPATH
            let drive = std::env::var("HOMEDRIVE").unwrap_or_default();
            let path = std::env::var("HOMEPATH").unwrap_or_default();
            if drive.is_empty() && path.is_empty() {
                Err(std::env::VarError::NotPresent)
            } else {
                Ok(format!("{}{}", drive, path))
            }
        })
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".agents").join("skills")
}

/// 从字符串中提取裸值（去除引号和空白）
fn extract_str_value(s: &str) -> String {
    s.trim().trim_matches('"').trim_matches('\'').to_string()
}

/// 解析 YAML 数组值，支持 ["a", "b"] 格式
fn extract_arr_value(s: &str) -> Vec<String> {
    let s = s.trim();
    if s.starts_with('[') && s.ends_with(']') {
        s[1..s.len() - 1]
            .split(',')
            .map(|item| item.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if !s.is_empty() && s != "[]" {
        vec![s.trim_matches('"').trim_matches('\'').to_string()]
    } else {
        Vec::new()
    }
}

/// 根据目录名和描述自动推断分类
fn infer_category(dir_name: &str, description: &str) -> String {
    let text = format!("{} {}", dir_name, description).to_lowercase();

    // UI 设计类关键词
    let ui_keywords = [
        "animate",
        "animation",
        "layout",
        "color",
        "colorize",
        "typeset",
        "typography",
        "bolder",
        "bold",
        "distill",
        "polish",
        "quieter",
        "delight",
        "adapt",
        "responsive",
        "design",
        "ui",
        "ux",
        "visual",
        "style",
        "theme",
        "icon",
        "font",
        "css",
        "critique",
        "audit",
        "shape",
        "impeccable",
        "motion",
        "transition",
    ];
    // 编码开发类关键词
    let coding_keywords = [
        "coding",
        "code",
        "backend",
        "frontend",
        "typescript",
        "javascript",
        "python",
        "client",
        "server",
        "api",
        "database",
        "test",
        "lint",
        "format",
        "build",
        "webpack",
        "vite",
        "react",
        "vue",
        "angular",
        "node",
        "rust",
        "java",
        "upgrade",
        "migration",
        "security",
        "auth",
    ];
    // 工具效率类关键词
    let tools_keywords = [
        "tool",
        "find",
        "search",
        "install",
        "manage",
        "optimize",
        "performance",
        "workflow",
        "productivity",
        "automation",
        "deploy",
        "ci",
        "cd",
        "git",
        "docker",
        "cloud",
        "azure",
        "aws",
        "infrastructure",
        "devops",
    ];
    // AI / 智能体类关键词
    let ai_keywords = [
        "ai",
        "llm",
        "gpt",
        "agent",
        "copilot",
        "prompt",
        "skill",
        "chat",
        "model",
        "inference",
        "embedding",
        "vector",
    ];
    // 文档写作类关键词
    let docs_keywords = [
        "doc",
        "document",
        "readme",
        "markdown",
        "write",
        "clarify",
        "copy",
        "content",
        "text",
        "blog",
        "diagram",
        "architecture",
    ];

    // 按优先级匹配（使用命中数量最多的分类）
    let categories: &[(&str, &[&str])] = &[
        ("AI 智能体", &ai_keywords),
        ("UI 设计", &ui_keywords),
        ("编码开发", &coding_keywords),
        ("文档写作", &docs_keywords),
        ("工具效率", &tools_keywords),
    ];

    let mut best = ("通用", 0usize);
    for (cat, keywords) in categories {
        let count = keywords.iter().filter(|kw| text.contains(**kw)).count();
        if count > best.1 {
            best = (cat, count);
        }
    }
    best.0.to_string()
}

/// 解析 SKILL.md 的 YAML frontmatter，提取关键字段
fn parse_skill_md(content: &str, dir_name: &str) -> SkillInfo {
    let mut name = dir_name.to_string();
    let mut description = String::new();
    let mut category = String::new();
    let mut applies_to: Vec<String> = Vec::new();
    let mut depends_on: Vec<String> = Vec::new();

    if content.starts_with("---") {
        let after_open = &content[3..];
        if let Some(end_pos) = after_open.find("\n---") {
            let frontmatter = &after_open[..end_pos];
            let mut in_metadata = false;

            for line in frontmatter.lines() {
                let trimmed = line.trim();

                if trimmed == "metadata:" {
                    in_metadata = true;
                    continue;
                }

                // 非缩进行是顶层字段
                if !line.starts_with(' ') && !line.starts_with('\t') {
                    in_metadata = false;
                    if let Some(rest) = trimmed.strip_prefix("name:") {
                        name = extract_str_value(rest);
                    } else if let Some(rest) = trimmed.strip_prefix("description:") {
                        description = extract_str_value(rest);
                    }
                } else if in_metadata {
                    // metadata 子字段
                    if let Some(rest) = trimmed.strip_prefix("category:") {
                        category = extract_str_value(rest);
                    } else if let Some(rest) = trimmed.strip_prefix("appliesTo:") {
                        applies_to = extract_arr_value(rest);
                    } else if let Some(rest) = trimmed.strip_prefix("dependsOn:") {
                        depends_on = extract_arr_value(rest);
                    }
                }
            }
        }
    }

    // category 为空时自动推断
    let final_category = if category.is_empty() {
        infer_category(dir_name, &description)
    } else {
        category
    };

    SkillInfo {
        name,
        description,
        category: final_category,
        dir_name: dir_name.to_string(),
        applies_to,
        depends_on,
        has_skill_file: true,
    }
}

/// 校验目录名安全性，防止路径穿越
fn validate_dir_name(dir_name: &str) -> Result<(), String> {
    if dir_name.is_empty()
        || dir_name.contains('/')
        || dir_name.contains('\\')
        || dir_name.contains("..")
        || dir_name.starts_with('.')
    {
        return Err("非法目录名，禁止路径穿越字符".to_string());
    }
    Ok(())
}

/// 扫描 ~/.agents/skills/ 目录，返回所有 skill 信息列表
#[tauri::command]
fn list_skills() -> Result<Vec<SkillInfo>, String> {
    let skills_dir = get_skills_dir();

    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&skills_dir).map_err(|e| e.to_string())?;
    let mut skills = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // 跳过隐藏目录
        if dir_name.starts_with('.') {
            continue;
        }

        let skill_file = path.join("SKILL.md");

        if skill_file.exists() {
            let content = fs::read_to_string(&skill_file).unwrap_or_default();
            skills.push(parse_skill_md(&content, &dir_name));
        } else {
            skills.push(SkillInfo {
                name: dir_name.clone(),
                description: "未找到 SKILL.md 文件".to_string(),
                category: "未分类".to_string(),
                dir_name,
                applies_to: Vec::new(),
                depends_on: Vec::new(),
                has_skill_file: false,
            });
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// 读取指定 skill 的 SKILL.md 原始内容
#[tauri::command]
fn get_skill_content(dir_name: String) -> Result<String, String> {
    validate_dir_name(&dir_name)?;

    let skills_dir = get_skills_dir();
    let skill_file = skills_dir.join(&dir_name).join("SKILL.md");

    if !skill_file.exists() {
        return Err(format!("'{}' 不存在 SKILL.md", dir_name));
    }

    fs::read_to_string(&skill_file).map_err(|e| e.to_string())
}

/// 通过 git clone 安装 skill（支持 GitHub / GitLab / Gitee）
#[tauri::command]
fn install_skill_git(url: String) -> Result<InstallResult, String> {
    let url = url.trim().to_string();

    // 仅允许可信来源，防止 shell 注入
    let allowed_prefixes = [
        "https://github.com/",
        "https://gitlab.com/",
        "https://gitee.com/",
    ];
    if !allowed_prefixes.iter().any(|p| url.starts_with(p)) {
        return Err("仅支持 GitHub、GitLab 或 Gitee 的 HTTPS URL".to_string());
    }

    // 从 URL 提取仓库名作为安装目录名
    let repo_name = url
        .split('/')
        .last()
        .unwrap_or("unknown-skill")
        .trim_end_matches(".git")
        .to_string();

    if repo_name.is_empty() || repo_name.contains("..") {
        return Err("无法从 URL 中解析仓库名".to_string());
    }

    let skills_dir = get_skills_dir();

    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir).map_err(|e| e.to_string())?;
    }

    let target = skills_dir.join(&repo_name);
    if target.exists() {
        return Err(format!("技能 '{}' 已存在，请先删除再重新安装", repo_name));
    }

    let output = Command::new("git")
        .args(["clone", &url, target.to_str().unwrap_or("")])
        .output()
        .map_err(|e| format!("执行 git 命令失败: {}", e))?;

    if output.status.success() {
        Ok(InstallResult {
            success: true,
            dir_name: repo_name,
            message: "安装成功".to_string(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("克隆失败: {}", stderr.trim()))
    }
}

/// 删除指定 skill 目录
#[tauri::command]
fn delete_skill(dir_name: String) -> Result<(), String> {
    validate_dir_name(&dir_name)?;

    let skills_dir = get_skills_dir();
    let target = skills_dir.join(&dir_name);

    if !target.exists() {
        return Err(format!("技能 '{}' 不存在", dir_name));
    }

    // 额外校验：目标必须在 skills_dir 之内（防止 symlink 攻击）
    let canonical_target = target.canonicalize().map_err(|e| e.to_string())?;
    let canonical_skills = skills_dir.canonicalize().map_err(|e| e.to_string())?;

    if !canonical_target.starts_with(&canonical_skills) {
        return Err("路径校验失败，操作已阻止".to_string());
    }

    fs::remove_dir_all(&target).map_err(|e| e.to_string())
}

/// 查询 AI 分析缓存
/// 若 content_hash 与存储值不符（SKILL.md 已更新），返回 null
#[tauri::command]
fn get_ai_cache(
    state: State<DbState>,
    dir_name: String,
    content_hash: String,
) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT data, content_hash FROM ai_cache WHERE dir_name = ?1",
        params![dir_name],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );
    match result {
        Ok((data, stored_hash)) => {
            if stored_hash == content_hash {
                Ok(Some(data))
            } else {
                Ok(None)
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 写入 AI 分析缓存（INSERT OR REPLACE）
#[tauri::command]
fn set_ai_cache(
    state: State<DbState>,
    dir_name: String,
    content_hash: String,
    data: String,
    timestamp: i64,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO ai_cache (dir_name, content_hash, data, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![dir_name, content_hash, data, timestamp],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// 删除指定技能的 AI 分析缓存
#[tauri::command]
fn delete_ai_cache(state: State<DbState>, dir_name: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM ai_cache WHERE dir_name = ?1",
        params![dir_name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取 skills 目录的完整路径（用于前端展示）
#[tauri::command]
fn get_skills_dir_path() -> String {
    get_skills_dir().to_string_lossy().to_string()
}

/// 读取 AI 配置文件（应用数据目录/ai_config.json）
/// 文件不存在时返回空对象 {}
#[tauri::command]
fn get_ai_config(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("ai_config.json");
    if !path.exists() {
        return Ok("{}".to_string());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// 写入 AI 配置文件（覆盖写入）
#[tauri::command]
fn set_ai_config(app: tauri::AppHandle, config: String) -> Result<(), String> {
    // 校验：必须是合法 JSON，防止写入损坏数据
    serde_json::from_str::<serde_json::Value>(&config)
        .map_err(|e| format!("无效的配置 JSON: {}", e))?;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join("ai_config.json"), config).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // ===== 初始化 AI 缓存数据库 =====
            let conn = init_db(app.handle()).expect("无法初始化 AI 缓存数据库");
            app.manage(DbState(Mutex::new(conn)));

            // ===== 系统托盘设置 =====
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let hide_item = MenuItem::with_id(app, "hide", "隐藏窗口", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出 Skills Hub", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &hide_item, &sep, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Skills Hub")
                .menu(&menu)
                // macOS 左键点击直接切换窗口显示，不弹菜单
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    // 左键单击：切换窗口显示/隐藏
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // ===== 关闭窗口时隐藏到托盘而非退出 =====
            let main_win = app.get_webview_window("main").unwrap();
            main_win.on_window_event(|event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // 拦截关闭请求，改为隐藏窗口
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            list_skills,
            get_skill_content,
            install_skill_git,
            delete_skill,
            get_skills_dir_path,
            get_ai_cache,
            set_ai_cache,
            delete_ai_cache,
            get_ai_config,
            set_ai_config,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
