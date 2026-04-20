<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  Search, Plus, RefreshCw, BookOpen, Loader2, AlertCircle, SlidersHorizontal, X, Settings
} from 'lucide-vue-next'
import { useSkillsStore } from './stores/skills'
import type { SkillInfo } from './types/skill'
import SkillCard from './components/SkillCard.vue'
import InstallModal from './components/InstallModal.vue'
import SkillDetailPanel from './components/SkillDetailPanel.vue'
import SettingsModal from './components/SettingsModal.vue'

const store = useSkillsStore()
const showInstall = ref(false)
const showSettings = ref(false)
const selectedSkill = ref<SkillInfo | null>(null)

function viewSkill(skill: SkillInfo) {
  selectedSkill.value = skill
}

function handleDeleteFromCard(skill: SkillInfo) {
  selectedSkill.value = skill
}

onMounted(() => {
  store.fetchSkills()
})
</script>

<template>
  <div class="h-screen flex flex-col overflow-hidden bg-bg">

    <!-- 顶部导航栏 -->
    <header class="shrink-0 flex items-center gap-4 px-6 py-3.5 border-b border-border bg-surface/80 backdrop-blur-sm z-10">
      <!-- Logo -->
      <div class="flex items-center gap-2.5 shrink-0">
        <div class="w-7 h-7 rounded-lg flex items-center justify-center bg-accent-bg border border-accent/30">
          <BookOpen :size="14" class="text-accent" />
        </div>
        <span class="font-display font-semibold text-text text-base tracking-tight">Skills Hub</span>
        <span class="text-xs text-text-muted bg-elevated px-1.5 py-0.5 rounded-md border border-border">
          {{ store.skills.length }}
        </span>
      </div>

      <!-- 搜索框 -->
      <div class="flex-1 relative max-w-md">
        <Search :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
        <input v-model="store.searchQuery" type="text" placeholder="搜索名称、描述、标签…" class="w-full pl-9 pr-9 py-2 rounded-lg text-sm text-text placeholder:text-text-muted bg-input border border-border outline-none focus:border-accent transition-all" />
        <button v-if="store.searchQuery" class="absolute right-2.5 top-1/2 -translate-y-1/2 p-0.5 rounded text-text-muted hover:text-text transition-colors" @click="store.searchQuery = ''">
          <X :size="13" />
        </button>
      </div>

      <!-- 右侧操作 -->
      <div class="flex items-center gap-2 shrink-0">
        <button class="p-2 rounded-lg border border-border text-text-muted hover:text-text hover:bg-elevated transition-colors" title="设置" @click="showSettings = true">
          <Settings :size="15" />
        </button>
        <button class="p-2 rounded-lg border border-border text-text-muted hover:text-text hover:bg-elevated transition-colors" title="刷新列表" @click="store.fetchSkills()">
          <RefreshCw :size="15" :class="{ 'animate-spin': store.loading }" />
        </button>
        <button class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-accent text-accent-content hover:bg-accent-hover transition-colors" @click="showInstall = true">
          <Plus :size="15" />
          安装技能
        </button>
      </div>
    </header>

    <!-- 分类过滤栏 -->
    <div class="shrink-0 flex items-center gap-1.5 px-6 py-2.5 border-b border-border overflow-x-auto no-scrollbar bg-surface/40">
      <SlidersHorizontal :size="13" class="text-text-muted shrink-0 mr-1" />
      <button v-for="cat in store.categories" :key="cat" class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium whitespace-nowrap transition-all border" :class="store.selectedCategory === cat
        ? 'bg-accent-bg border-accent/40 text-accent'
        : 'border-transparent text-text-muted hover:text-text hover:bg-elevated'" @click="store.selectedCategory = cat">
        {{ cat }}
        <span class="px-1.5 py-0.5 rounded text-[10px] font-semibold tabular-nums" :class="store.selectedCategory === cat ? 'bg-accent text-accent-content' : 'bg-elevated text-text-muted'">
          {{ store.categoryCount[cat] ?? 0 }}
        </span>
      </button>
    </div>

    <!-- 主内容区 -->
    <main class="flex-1 overflow-y-auto p-6">
      <!-- 加载 -->
      <div v-if="store.loading" class="flex flex-col items-center justify-center h-64 gap-3 text-text-muted">
        <Loader2 :size="28" class="animate-spin text-accent" />
        <p class="text-sm">扫描技能目录中…</p>
      </div>

      <!-- 错误 -->
      <div v-else-if="store.error" class="flex items-start gap-3 p-4 rounded-xl bg-danger-bg border border-[oklch(0.35_0.08_22)] text-danger max-w-lg mx-auto mt-12">
        <AlertCircle :size="18" class="shrink-0 mt-0.5" />
        <div>
          <p class="text-sm font-medium">加载失败</p>
          <p class="text-xs mt-1 opacity-80">{{ store.error }}</p>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else-if="store.filteredSkills.length === 0" class="flex flex-col items-center justify-center h-64 gap-4 text-center">
        <div class="w-14 h-14 rounded-2xl bg-elevated flex items-center justify-center border border-border">
          <BookOpen :size="24" class="text-text-muted" />
        </div>
        <div>
          <p class="text-sm font-medium text-text-dim">
            {{ store.searchQuery ? '没有匹配的技能' : '暂无技能' }}
          </p>
          <p class="text-xs text-text-muted mt-1">
            {{ store.searchQuery ? '尝试修改关键词或清除过滤' : `安装目录：${store.skillsDir || '~/.agents/skills'}` }}
          </p>
        </div>
        <button v-if="!store.searchQuery" class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-accent text-[oklch(0.12_0.02_65)] hover:bg-accent-hover transition-colors" @click="showInstall = true">
          <Plus :size="14" />
          安装第一个技能
        </button>
      </div>

      <!-- 技能网格 -->
      <div v-else class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));">
        <SkillCard v-for="skill in store.filteredSkills" :key="skill.dirName" :skill="skill" @view="viewSkill" @delete="handleDeleteFromCard" />
      </div>

      <!-- 底部统计 -->
      <div v-if="!store.loading && store.filteredSkills.length > 0" class="mt-8 pt-6 border-t border-border text-center text-xs text-text-muted">
        共 {{ store.filteredSkills.length }} 个技能
        <template v-if="store.selectedCategory !== '全部' || store.searchQuery">
          （全部 {{ store.skills.length }} 个中筛选）
        </template>
      </div>
    </main>
  </div>

  <!-- 弹窗层 -->
  <Transition name="modal">
    <InstallModal v-if="showInstall" @close="showInstall = false" @installed="store.fetchSkills()" />
  </Transition>

  <Transition name="panel">
    <SkillDetailPanel v-if="selectedSkill" :skill="selectedSkill" @close="selectedSkill = null" @delete="handleDeleteFromCard" />
  </Transition>

  <SettingsModal v-model:visible="showSettings" />
</template>

<style>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}

.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
