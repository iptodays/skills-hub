/**
 * AI 分析 Pinia Store
 * 管理 Claude API 配置、各技能的分析状态及缓存加载
 * 配置持久化到应用数据目录的 ai_config.json（通过 Tauri 命令读写）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { analyzeSkill, type AnalysisConfig } from '../services/aiAnalysis'
import { getCached, clearCached, type AiAnalysis } from '../services/aiCache'

/** API 配置结构 */
export interface AiSettings {
  apiKey: string
  baseUrl: string
  model: string
}

const DEFAULTS: AiSettings = {
  apiKey: '',
  baseUrl: '',
  model: 'claude-3-haiku-20240307',
}

export const useAiStore = defineStore('ai', () => {
  /** API 配置（初始为默认值，loadConfig 后填充） */
  const settings = ref<AiSettings>({ ...DEFAULTS })
  const configLoaded = ref(false)

  /** 是否已配置有效的 API Key */
  const hasApiKey = computed(() => !!settings.value.apiKey.trim())

  /** 从文件加载配置（应用启动时调用一次） */
  async function loadConfig() {
    try {
      const raw = await invoke<string>('get_ai_config')
      const parsed = JSON.parse(raw)
      settings.value = { ...DEFAULTS, ...parsed }
    } catch { /* 文件不存在或损坏时使用默认值 */ }
    configLoaded.value = true
  }

  /** 将配置写入文件 */
  async function saveSettings(patch: Partial<AiSettings>) {
    settings.value = { ...settings.value, ...patch }
    await invoke('set_ai_config', { config: JSON.stringify(settings.value) })
  }

  /** 按 dirName 存储各技能的分析结果 */
  const analysisMap = ref<Record<string, AiAnalysis>>({})

  /** 各技能的加载状态 */
  const loadingMap = ref<Record<string, boolean>>({})

  /** 各技能的错误信息 */
  const errorMap = ref<Record<string, string>>({})

  /**
   * 从 SQLite 缓存加载分析结果（异步）
   * 在组件挂载或切换技能时调用，避免不必要的 API 请求
   */
  async function loadFromCache(dirName: string, content: string) {
    const cached = await getCached(dirName, content)
    if (cached) analysisMap.value[dirName] = cached
  }

  /**
   * 调用 Claude API 分析技能内容
   * 内部已实现缓存判断，命中缓存时不发起网络请求
   */
  async function analyze(dirName: string, content: string) {
    if (!hasApiKey.value) {
      errorMap.value[dirName] = '请先配置 Claude API Key'
      return
    }

    loadingMap.value[dirName] = true
    delete errorMap.value[dirName]

    const cfg: AnalysisConfig = {
      apiKey: settings.value.apiKey,
      baseUrl: settings.value.baseUrl,
      model: settings.value.model,
    }

    try {
      const result = await analyzeSkill(dirName, content, cfg)
      analysisMap.value[dirName] = result
    } catch (e) {
      errorMap.value[dirName] = String(e)
    } finally {
      loadingMap.value[dirName] = false
    }
  }

  /** 清除指定技能的缓存及内存分析结果，用于强制重新分析 */
  async function clearAnalysis(dirName: string) {
    await clearCached(dirName)
    delete analysisMap.value[dirName]
    delete errorMap.value[dirName]
  }

  return {
    settings,
    configLoaded,
    hasApiKey,
    loadConfig,
    saveSettings,
    analysisMap,
    loadingMap,
    errorMap,
    loadFromCache,
    analyze,
    clearAnalysis,
  }
})
