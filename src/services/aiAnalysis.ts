/**
 * AI 技能分析服务
 * 调用 Claude API（或兼容接口）对 SKILL.md 进行深度分析
 * 所有结果自动写入本地缓存，重复请求直接命中缓存
 */

import { getCached, setCached, type AiAnalysis } from './aiCache'

/** AI 接入配置 */
export interface AnalysisConfig {
  /** Claude API Key（必填）*/
  apiKey: string
  /** 自定义 Base URL，留空则使用 Anthropic 官方接口 */
  baseUrl: string
  /** 模型名称 */
  model: string
}

/** Claude Messages API 请求 */
async function callClaude(
  systemPrompt: string,
  userContent: string,
  cfg: AnalysisConfig,
): Promise<string> {
  const base = (cfg.baseUrl || 'https://api.anthropic.com').replace(/\/$/, '')
  const resp = await fetch(`${base}/v1/messages`, {
    method: 'POST',
    headers: {
      'x-api-key': cfg.apiKey,
      'anthropic-version': '2023-06-01',
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      model: cfg.model || 'claude-3-haiku-20240307',
      max_tokens: 1200,
      system: systemPrompt,
      messages: [{ role: 'user', content: userContent }],
    }),
  })

  if (!resp.ok) {
    let msg = `HTTP ${resp.status}`
    try {
      const data = await resp.json()
      msg += ': ' + (data?.error?.message ?? JSON.stringify(data))
    } catch { /* ignore */ }
    throw new Error(msg)
  }

  const data = await resp.json()
  return data.content?.[0]?.text ?? ''
}

const SYSTEM_PROMPT = `你是一个 AI Agent 技能分析助手。
分析用户提供的技能文档，严格以 JSON 格式返回，不要包含 markdown 代码块标记或任何额外文字。`

/**
 * 分析指定技能
 * 优先读取本地缓存，缓存命中则直接返回，避免重复调用 API
 */
export async function analyzeSkill(
  dirName: string,
  content: string,
  cfg: AnalysisConfig,
): Promise<AiAnalysis> {
  // 优先查询 SQLite 缓存，命中则无需请求 API
  const cached = await getCached(dirName, content)
  if (cached) return cached

  const userMsg = `请分析以下 SKILL.md 文档：

${content.slice(0, 6000)}

以 JSON 格式返回，字段说明：
{
  "summary": "一句话描述该技能核心用途，不超过 25 字",
  "scenarios": ["触发/使用场景1", "场景2", "场景3"],
  "recommendations": ["推荐搭配的其他技能或工具名（只写名称）"],
  "examples": ["具体调用指令示例1", "具体调用指令示例2"]
}`

  const raw = await callClaude(SYSTEM_PROMPT, userMsg, cfg)

  // 提取 JSON（防止 Claude 在 JSON 前后附加说明文字）
  let parsed: Pick<AiAnalysis, 'summary' | 'scenarios' | 'recommendations' | 'examples'>
  try {
    const match = raw.match(/\{[\s\S]*\}/)
    if (!match) throw new Error('返回格式无效')
    parsed = JSON.parse(match[0])
  } catch {
    throw new Error('AI 返回解析失败: ' + raw.slice(0, 300))
  }

  // 写入 SQLite 缓存并返回
  return await setCached(dirName, content, parsed)
}
