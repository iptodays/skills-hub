/**
 * AI 分析结果缓存层
 * 数据持久化到 SQLite（应用数据目录/ai_cache.db），通过 Tauri 命令读写
 * 以内容哈希作为缓存失效依据，SKILL.md 内容变更时自动失效
 */

import { invoke } from '@tauri-apps/api/core'

/** AI 分析结果结构 */
export interface AiAnalysis {
  /** 一句话摘要 */
  summary: string
  /** 使用场景列表 */
  scenarios: string[]
  /** 推荐搭配的其他技能或工具 */
  recommendations: string[]
  /** 具体使用示例 */
  examples: string[]
  /** 分析时间戳（毫秒） */
  timestamp: number
  /** 内容哈希，用于判断缓存是否仍然有效 */
  contentHash: string
}

/**
 * djb2 哈希算法
 * 生成 SKILL.md 内容指纹，用于检测文件是否已更新
 */
function hashContent(str: string): string {
  let h = 5381
  for (let i = 0; i < str.length; i++) {
    h = ((h << 5) + h) ^ str.charCodeAt(i)
    h = h >>> 0
  }
  return h.toString(16)
}

/**
 * 从 SQLite 读取缓存
 * Rust 侧同时校验 content_hash，SKILL.md 已更新则返回 null
 */
export async function getCached(dirName: string, content: string): Promise<AiAnalysis | null> {
  const contentHash = hashContent(content)
  try {
    const raw = await invoke<string | null>('get_ai_cache', { dirName, contentHash })
    if (!raw) return null
    return JSON.parse(raw) as AiAnalysis
  } catch {
    return null
  }
}

/**
 * 写入分析结果到 SQLite
 * 自动附加时间戳和内容哈希后存储
 */
export async function setCached(
  dirName: string,
  content: string,
  result: Pick<AiAnalysis, 'summary' | 'scenarios' | 'recommendations' | 'examples'>,
): Promise<AiAnalysis> {
  const entry: AiAnalysis = {
    ...result,
    timestamp: Date.now(),
    contentHash: hashContent(content),
  }
  await invoke('set_ai_cache', {
    dirName,
    contentHash: entry.contentHash,
    data: JSON.stringify(entry),
    timestamp: entry.timestamp,
  })
  return entry
}

/** 删除指定技能的缓存记录 */
export async function clearCached(dirName: string): Promise<void> {
  await invoke('delete_ai_cache', { dirName })
}
