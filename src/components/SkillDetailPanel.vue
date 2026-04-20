<script setup lang="ts">
import { ref, watch, computed, reactive } from 'vue'
import { marked } from 'marked'
import {
  X, Trash2, Tag, Link, FolderOpen, ChevronRight, Loader2, AlertCircle,
  Sparkles, Settings, RefreshCw, Clock, ChevronDown,
} from 'lucide-vue-next'
import type { SkillInfo } from '../types/skill'
import { useSkillsStore } from '../stores/skills'
import { useAiStore } from '../stores/ai'

const props = defineProps<{ skill: SkillInfo }>()
const emit = defineEmits<{
  close: []
  delete: [skill: SkillInfo]
}>()

const store = useSkillsStore()
const aiStore = useAiStore()

const mdContent = ref('')
const mdLoading = ref(false)
const mdError = ref('')
const showDeleteConfirm = ref(false)

/** 标签页：文档 / AI 分析 */
const activeTab = ref<'doc' | 'ai'>('doc')

/** API Key 配置面板是否展开 */
const showAiSettings = ref(false)

/** 配置草稿，保存时才写入 store */
const settingsDraft = reactive({ apiKey: '', baseUrl: '', model: '' })

/** 展开配置面板时同步当前值 */
watch(showAiSettings, (v) => {
  if (v) {
    settingsDraft.apiKey = aiStore.settings.apiKey
    settingsDraft.baseUrl = aiStore.settings.baseUrl
    settingsDraft.model = aiStore.settings.model
  }
})

function saveSettings() {
  aiStore.saveSettings({ ...settingsDraft })
  showAiSettings.value = false
}

/** 当前技能的 AI 分析结果 */
const aiResult = computed(() => aiStore.analysisMap[props.skill.dirName])
const aiLoading = computed(() => !!aiStore.loadingMap[props.skill.dirName])
const aiError = computed(() => aiStore.errorMap[props.skill.dirName] ?? '')

async function runAnalysis() {
  await aiStore.analyze(props.skill.dirName, mdContent.value)
}

/** 清除缓存并重新分析 */
async function reAnalyze() {
  aiStore.clearAnalysis(props.skill.dirName)
  await aiStore.analyze(props.skill.dirName, mdContent.value)
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleString('zh-CN', {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
  })
}

/** 剥离 YAML frontmatter，只返回正文 Markdown */
function stripFrontmatter(raw: string): string {
  if (!raw.startsWith('---')) return raw
  const rest = raw.slice(3)
  const end = rest.indexOf('\n---')
  if (end === -1) return raw
  return rest.slice(end + 4).replace(/^\n/, '')
}

/** 渲染 Markdown 为 HTML */
const htmlContent = computed(() => {
  if (!mdContent.value) return ''
  return marked.parse(stripFrontmatter(mdContent.value)) as string
})

/** 分类徽章样式 */
function categoryStyle(category: string): string {
  const map: Record<string, string> = {
    'Core': 'bg-cat-core/10 text-cat-core border-cat-core/20',
    'Frontend': 'bg-cat-frontend/10 text-cat-frontend border-cat-frontend/20',
    'Desktop': 'bg-cat-desktop/10 text-cat-desktop border-cat-desktop/20',
    'Design': 'bg-cat-design/10  text-cat-design  border-cat-design/20',
    'Backend': 'bg-cat-backend/10 text-cat-backend border-cat-backend/20',
  }
  return map[category] ?? 'bg-cat-default/10 text-cat-default border-cat-default/20'
}

/** 切换技能时加载内容，并尝试从本地缓存恢复 AI 分析结果 */
watch(() => props.skill, async (skill) => {
  activeTab.value = 'doc'
  showDeleteConfirm.value = false
  if (!skill.hasSkillFile) {
    mdContent.value = ''
    mdError.value = '该技能目录下没有 SKILL.md 文件'
    return
  }
  mdLoading.value = true
  mdError.value = ''
  try {
    mdContent.value = await store.getSkillContent(skill.dirName)
    // 文档加载完成后，尝试从本地缓存恢复 AI 分析（无网络请求）
    aiStore.loadFromCache(skill.dirName, mdContent.value)
  } catch (e) {
    mdError.value = String(e)
  } finally {
    mdLoading.value = false
  }
}, { immediate: true })

async function confirmDelete() {
  try {
    await store.deleteSkill(props.skill.dirName)
    emit('close')
  } catch (e) {
    mdError.value = String(e)
    showDeleteConfirm.value = false
  }
}
</script>

<template>
  <!-- 遮罩 -->
  <div class="fixed inset-0 z-50 flex items-end sm:items-center justify-end bg-black/50 backdrop-blur-sm" @click.self="emit('close')">
    <!-- 面板：从右侧滑入 -->
    <div class="w-full sm:w-140 h-full bg-surface border-l border-border flex flex-col shadow-2xl">
      <!-- 头部 -->
      <div class="flex items-start gap-4 px-6 py-5 border-b border-border shrink-0">
        <div class="flex-1 min-w-0">
          <!-- 分类徽章 -->
          <span class="inline-flex items-center px-2 py-0.5 rounded-md text-xs font-medium border mb-2" :class="categoryStyle(skill.category)">
            {{ skill.category || '未分类' }}
          </span>

          <!-- 技能名 -->
          <h2 class="font-display font-semibold text-lg text-text leading-tight truncate">
            {{ skill.name }}
          </h2>

          <!-- 目录名 -->
          <p class="text-xs text-text-muted mt-0.5 flex items-center gap-1">
            <FolderOpen :size="11" />
            {{ skill.dirName }}
          </p>
        </div>

        <button class="p-1.5 rounded-lg text-text-muted hover:text-text hover:bg-elevated transition-colors shrink-0" @click="emit('close')">
          <X :size="18" />
        </button>
      </div>

      <!-- 元信息标签区 -->
      <div class="px-6 py-3 border-b border-border flex flex-wrap gap-2 shrink-0">
        <!-- appliesTo 标签 -->
        <span v-for="tag in skill.appliesTo" :key="tag" class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs
                 bg-elevated text-text-dim border border-border">
          <Tag :size="10" class="opacity-60" />
          {{ tag }}
        </span>

        <!-- dependsOn -->
        <template v-if="skill.dependsOn.length > 0">
          <div class="w-px h-4 bg-border self-center" />
          <span v-for="dep in skill.dependsOn" :key="dep" class="inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs
                   bg-input text-text-muted border border-border">
            <Link :size="10" class="opacity-60" />
            {{ dep }}
          </span>
        </template>
      </div>

      <!-- 标签页切换 -->
      <div class="flex border-b border-border shrink-0 px-6">
        <button v-for="tab in [{ id: 'doc', label: '文档' }, { id: 'ai', label: 'AI 分析' }]" :key="tab.id" class="flex items-center gap-1.5 px-1 py-3 mr-6 text-sm font-medium border-b-2 -mb-px transition-colors" :class="activeTab === tab.id
          ? 'border-accent text-accent'
          : 'border-transparent text-text-muted hover:text-text'" @click="activeTab = (tab.id as 'doc' | 'ai')">
          <Sparkles v-if="tab.id === 'ai'" :size="13" />
          {{ tab.label }}
          <!-- AI 分析已完成的小圆点指示 -->
          <span v-if="tab.id === 'ai' && aiResult && !aiLoading" class="w-1.5 h-1.5 rounded-full bg-accent" />
        </button>
      </div>

      <!-- 文档标签页 -->
      <div v-if="activeTab === 'doc'" class="flex-1 overflow-y-auto px-6 py-5">
        <!-- 加载中 -->
        <div v-if="mdLoading" class="flex items-center justify-center h-32 text-text-muted gap-2">
          <Loader2 :size="18" class="animate-spin" />
          <span class="text-sm">加载中…</span>
        </div>

        <!-- 错误 -->
        <div v-else-if="mdError" class="flex items-start gap-2 p-4 rounded-lg bg-danger-bg border border-danger/20 text-danger text-sm">
          <AlertCircle :size="16" class="shrink-0 mt-0.5" />
          {{ mdError }}
        </div>

        <!-- Markdown 渲染 -->
        <div v-else class="markdown-body" v-html="htmlContent" />
      </div>

      <!-- AI 分析标签页 -->
      <div v-else class="flex-1 overflow-y-auto px-5 py-4 flex flex-col gap-4">

        <!-- API Key 配置（可折叠） -->
        <div class="rounded-xl border border-border overflow-hidden">
          <button class="w-full flex items-center justify-between px-4 py-3 text-sm transition-colors hover:bg-elevated/50" @click="showAiSettings = !showAiSettings">
            <div class="flex items-center gap-2 text-text-muted">
              <Settings :size="13" />
              <span>Claude API 配置</span>
              <span class="text-[11px] px-1.5 py-0.5 rounded-md" :class="aiStore.hasApiKey
                ? 'bg-success-bg text-success'
                : 'bg-accent/10 text-accent'">{{ aiStore.hasApiKey ? '已配置' : '未配置' }}</span>
            </div>
            <ChevronDown :size="13" class="text-text-muted transition-transform" :class="showAiSettings ? 'rotate-180' : ''" />
          </button>
          <div v-if="showAiSettings" class="px-4 pb-4 flex flex-col gap-3 border-t border-border bg-elevated/30">
            <label class="flex flex-col gap-1.5 mt-3">
              <span class="text-xs text-text-muted">API Key</span>
              <input v-model="settingsDraft.apiKey" type="password" placeholder="sk-ant-..." class="text-sm bg-surface border border-border rounded-lg px-3 py-2 text-text placeholder:text-text-muted/50 outline-none focus:border-accent transition-colors" />
            </label>
            <label class="flex flex-col gap-1.5">
              <span class="text-xs text-text-muted">Base URL <span class="opacity-50">（留空使用官方接口）</span></span>
              <input v-model="settingsDraft.baseUrl" type="text" placeholder="https://api.anthropic.com" class="text-sm bg-surface border border-border rounded-lg px-3 py-2 text-text placeholder:text-text-muted/50 outline-none focus:border-accent transition-colors" />
            </label>
            <label class="flex flex-col gap-1.5">
              <span class="text-xs text-text-muted">模型</span>
              <input v-model="settingsDraft.model" type="text" placeholder="claude-3-haiku-20240307" class="text-sm bg-surface border border-border rounded-lg px-3 py-2 text-text placeholder:text-text-muted/50 outline-none focus:border-accent transition-colors" />
            </label>
            <button class="self-end px-4 py-1.5 rounded-lg text-xs font-medium bg-accent text-accent-text hover:opacity-90 transition-opacity" @click="saveSettings">
              保存
            </button>
          </div>
        </div>

        <!-- 未分析入口 -->
        <div v-if="!aiResult && !aiLoading && !aiError" class="flex-1 flex flex-col items-center justify-center gap-3 py-10">
          <div class="w-12 h-12 rounded-2xl bg-accent/10 flex items-center justify-center">
            <Sparkles :size="22" class="text-accent opacity-80" />
          </div>
          <p class="text-sm text-text-muted text-center max-w-50 leading-relaxed">
            AI 深度分析技能用途、场景与使用示例
          </p>
          <button :disabled="!aiStore.hasApiKey || mdLoading" class="mt-1 px-6 py-2 rounded-xl text-sm font-medium bg-accent text-accent-text hover:opacity-90 transition-opacity disabled:opacity-40 disabled:cursor-not-allowed" @click="runAnalysis">
            开始分析
          </button>
          <p v-if="!aiStore.hasApiKey" class="text-xs text-accent/60">
            请先展开上方配置 API Key
          </p>
        </div>

        <!-- 分析中 -->
        <div v-if="aiLoading" class="flex-1 flex flex-col items-center justify-center gap-3 py-10 text-text-muted">
          <Loader2 :size="24" class="animate-spin text-accent" />
          <p class="text-sm">分析中，请稍候…</p>
        </div>

        <!-- 错误 -->
        <div v-if="aiError && !aiLoading" class="flex items-start gap-2 p-4 rounded-xl bg-danger-bg border border-danger/20 text-danger text-sm">
          <AlertCircle :size="15" class="shrink-0 mt-0.5" />
          <span class="flex-1">{{ aiError }}</span>
          <button class="text-xs underline opacity-70 hover:opacity-100 shrink-0" @click="runAnalysis">重试</button>
        </div>

        <!-- 分析结果 -->
        <template v-if="aiResult && !aiLoading">
          <!-- 摘要 -->
          <div class="rounded-xl bg-accent/5 border border-accent/10 px-4 py-3.5">
            <p class="text-[11px] font-semibold tracking-widest text-accent/60 mb-1.5">SUMMARY</p>
            <p class="text-sm text-text leading-relaxed">{{ aiResult.summary }}</p>
          </div>

          <!-- 使用场景 -->
          <div>
            <p class="text-[11px] font-semibold tracking-widest text-text-muted mb-2.5">使用场景</p>
            <ol class="flex flex-col gap-2">
              <li v-for="(s, i) in aiResult.scenarios" :key="i" class="flex items-start gap-2.5 text-sm text-text-dim">
                <span class="shrink-0 w-5 h-5 rounded-full bg-elevated border border-border flex items-center justify-center text-[11px] text-text-muted font-medium">{{ i + 1 }}</span>
                <span class="leading-relaxed pt-0.5">{{ s }}</span>
              </li>
            </ol>
          </div>

          <!-- 推荐搭配 -->
          <div v-if="aiResult.recommendations.length">
            <p class="text-[11px] font-semibold tracking-widest text-text-muted mb-2.5">推荐搭配</p>
            <div class="flex flex-wrap gap-2">
              <span v-for="r in aiResult.recommendations" :key="r" class="px-3 py-1 rounded-full text-xs bg-elevated border border-border text-text-dim">{{ r }}</span>
            </div>
          </div>

          <!-- 使用示例 -->
          <div v-if="aiResult.examples.length">
            <p class="text-[11px] font-semibold tracking-widest text-text-muted mb-2.5">使用示例</p>
            <ul class="flex flex-col gap-2">
              <li v-for="(ex, i) in aiResult.examples" :key="i" class="text-sm text-text-dim bg-elevated rounded-xl px-3.5 py-2.5 leading-relaxed">{{ ex }}</li>
            </ul>
          </div>

          <!-- 时间戳 + 重新分析 -->
          <div class="flex items-center justify-between text-xs text-text-muted pt-1 border-t border-border mt-1">
            <span class="flex items-center gap-1.5">
              <Clock :size="11" />
              {{ formatTime(aiResult.timestamp) }} · 已缓存
            </span>
            <button class="flex items-center gap-1 hover:text-text transition-colors" @click="reAnalyze">
              <RefreshCw :size="11" />
              重新分析
            </button>
          </div>
        </template>

      </div>

      <!-- 底部操作 -->
      <div class="px-6 py-4 border-t border-border shrink-0">
        <!-- 删除确认 -->
        <div v-if="showDeleteConfirm" class="flex flex-col gap-2.5 p-3.5 rounded-lg bg-danger-bg border border-[oklch(0.35_0.08_22)] mb-3">
          <div class="flex items-start gap-2">
            <AlertCircle :size="15" class="text-danger shrink-0 mt-0.5" />
            <div class="flex-1 space-y-1.5">
              <p class="text-sm font-medium text-danger">确认删除 <strong>{{ skill.name }}</strong>？</p>
              <ul class="text-xs text-danger/80 space-y-0.5 leading-relaxed">
                <li>• 技能目录 <code class="opacity-80">~/.agents/skills/{{ skill.dirName }}</code> 将被永久删除</li>
                <li>• Agent 将立即无法加载和使用该技能</li>
                <li>• 已引用此技能的 prompt 或工作流可能失效</li>
                <li>• 此操作不可撤销，如需恢复须重新安装</li>
              </ul>
            </div>
          </div>
          <div class="flex justify-end gap-2">
            <button class="px-3 py-1.5 rounded-lg text-xs font-medium border border-border text-text-dim hover:bg-elevated transition-colors" @click="showDeleteConfirm = false">
              取消
            </button>
            <button class="px-3 py-1.5 rounded-lg text-xs font-medium bg-danger text-white hover:opacity-90 transition-opacity" @click="confirmDelete">
              确认删除
            </button>
          </div>
        </div>

        <!-- 操作按钮行 -->
        <div v-else class="flex items-center justify-between">
          <div class="text-xs text-text-muted flex items-center gap-1">
            <ChevronRight :size="12" />
            <span>{{ store.skillsDir }}/{{ skill.dirName }}</span>
          </div>
          <button class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium
                   border border-[oklch(0.35_0.08_22)] text-danger hover:bg-danger-bg transition-colors" @click="showDeleteConfirm = true">
            <Trash2 :size="14" />
            删除技能
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
