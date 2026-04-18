<script setup lang="ts">
import { ref } from 'vue'
import { X, GitBranch, Loader2, CheckCircle, AlertCircle } from 'lucide-vue-next'
import { useSkillsStore } from '../stores/skills'

const emit = defineEmits<{
  close: []
  installed: [dirName: string]
}>()

const store = useSkillsStore()

const url = ref('')
const installing = ref(false)
const result = ref<{ ok: boolean; message: string } | null>(null)
/** 是否处于二次确认阶段 */
const pendingConfirm = ref(false)

/** 验证 URL 格式 */
const isValidUrl = (u: string) => {
  const trimmed = u.trim()
  return (
    trimmed.startsWith('https://github.com/') ||
    trimmed.startsWith('https://gitlab.com/') ||
    trimmed.startsWith('https://gitee.com/')
  )
}

/** 第一步：展示确认说明 */
function requestInstall() {
  if (!isValidUrl(url.value)) return
  pendingConfirm.value = true
}

/** 取消确认，返回输入阶段 */
function cancelConfirm() {
  pendingConfirm.value = false
}

/** 第二步：用户确认后执行安装 */
async function handleInstall() {
  installing.value = true
  result.value = null

  try {
    const res = await store.installSkillGit(url.value.trim())
    result.value = { ok: true, message: `'${res.dirName}' 安装成功` }
    // 刷新列表
    await store.fetchSkills()
    emit('installed', res.dirName)
    // 延迟关闭
    setTimeout(() => emit('close'), 1200)
  } catch (e) {
    result.value = { ok: false, message: String(e) }
    pendingConfirm.value = false
  } finally {
    installing.value = false
  }
}

/** 示例 URL 点击填入 */
function fillExample(example: string) {
  url.value = example
}
</script>

<template>
  <!-- 遮罩层 -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4" style="background: oklch(0 0 0 / 0.6); backdrop-filter: blur(4px);" @click.self="emit('close')">
    <div class="w-full max-w-md bg-surface border border-border rounded-2xl shadow-2xl overflow-hidden" style="box-shadow: 0 25px 50px oklch(0 0 0 / 0.5), 0 0 0 1px var(--color-border);">
      <!-- 头部 -->
      <div class="flex items-center justify-between px-6 pt-6 pb-4 border-b border-border">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-accent-bg">
            <GitBranch :size="18" class="text-accent" />
          </div>
          <div>
            <h2 class="font-display font-semibold text-text text-base leading-none">安装技能</h2>
            <p class="text-xs text-text-muted mt-0.5">从 Git 仓库克隆安装</p>
          </div>
        </div>
        <button class="p-1.5 rounded-lg text-text-muted hover:text-text hover:bg-elevated transition-colors" @click="emit('close')">
          <X :size="18" />
        </button>
      </div>

      <!-- 内容 -->
      <div class="p-6 space-y-5">
        <!-- URL 输入 -->
        <div class="space-y-2">
          <label class="text-sm font-medium text-text-dim">仓库地址</label>
          <input v-model="url" type="url" placeholder="https://github.com/user/skill-name" class="w-full px-4 py-2.5 rounded-lg text-sm text-text placeholder:text-text-muted
                   border border-border bg-input outline-none
                   focus:border-accent focus:ring-1 focus:ring-accent/30 transition-colors" :disabled="installing" @keydown.enter="handleInstall" />
          <p class="text-xs text-text-muted">支持 GitHub、GitLab、Gitee 的 HTTPS 链接</p>
        </div>

        <!-- 示例快填 -->
        <div class="space-y-1.5">
          <p class="text-xs font-medium text-text-muted">快速示例</p>
          <div class="flex flex-wrap gap-2">
            <button v-for="eg in ['https://github.com/user/impeccable', 'https://github.com/user/coding-standards']" :key="eg" class="px-2.5 py-1 rounded-md text-xs border border-border text-text-dim
                     hover:border-accent hover:text-accent bg-elevated transition-colors truncate max-w-full" @click="fillExample(eg)">
              {{ eg.replace('https://github.com/', '') }}
            </button>
          </div>
        </div>

        <!-- 安装结果提示 -->
        <div v-if="result" class="flex items-start gap-2.5 p-3 rounded-lg text-sm" :class="result.ok
          ? 'bg-success-bg border border-[oklch(0.35_0.08_145)] text-success'
          : 'bg-danger-bg border border-[oklch(0.35_0.08_22)] text-danger'">
          <CheckCircle v-if="result.ok" :size="16" class="shrink-0 mt-0.5" />
          <AlertCircle v-else :size="16" class="shrink-0 mt-0.5" />
          <span class="leading-relaxed">{{ result.message }}</span>
        </div>

        <!-- 二次确认说明 -->
        <div v-if="pendingConfirm && !result" class="rounded-lg border border-[oklch(0.32_0.08_65)] bg-[oklch(0.17_0.04_65)] p-3.5 space-y-2">
          <p class="text-sm font-medium text-[oklch(0.80_0.14_65)]">确认安装此技能？</p>
          <ul class="text-xs text-text-dim space-y-1 leading-relaxed list-none">
            <li>• 仓库将被克隆到 <code class="text-text-muted">~/.agents/skills/</code> 目录</li>
            <li>• 安装后 Agent 在当前对话中即可加载并使用该技能</li>
            <li>• 来自未知来源的技能可能含有不安全指令，请确认仓库可信</li>
          </ul>
        </div>
      </div>

      <!-- 底部按钮 -->
      <div class="flex justify-end gap-3 px-6 pb-6">
        <!-- 未进入确认阶段 -->
        <template v-if="!pendingConfirm">
          <button class="px-4 py-2 rounded-lg text-sm font-medium text-text-dim
                   border border-border hover:bg-elevated hover:text-text transition-colors" @click="emit('close')">
            取消
          </button>
          <button class="px-4 py-2 rounded-lg text-sm font-medium transition-all flex items-center gap-2
                   disabled:opacity-40 disabled:cursor-not-allowed" :class="isValidUrl(url)
                    ? 'bg-accent text-[oklch(0.12_0.02_65)] hover:bg-accent-hover'
                    : 'bg-elevated text-text-muted cursor-not-allowed'" :disabled="!isValidUrl(url)" @click="requestInstall">
            <GitBranch :size="14" />
            安装
          </button>
        </template>

        <!-- 确认阶段 -->
        <template v-else>
          <button class="px-4 py-2 rounded-lg text-sm font-medium text-text-dim
                   border border-border hover:bg-elevated hover:text-text transition-colors" :disabled="installing" @click="cancelConfirm">
            返回
          </button>
          <button class="px-4 py-2 rounded-lg text-sm font-medium transition-all flex items-center gap-2
                   disabled:opacity-40 disabled:cursor-not-allowed
                   bg-accent text-[oklch(0.12_0.02_65)] hover:bg-accent-hover" :disabled="installing" @click="handleInstall">
            <Loader2 v-if="installing" :size="14" class="animate-spin" />
            <GitBranch v-else :size="14" />
            {{ installing ? '安装中…' : '确认安装' }}
          </button>
        </template>
      </div>
    </div>
  </div>
</template>
