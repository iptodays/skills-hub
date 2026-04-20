<script setup lang="ts">
import type { SkillInfo } from '../types/skill'
import { Eye, Trash2, Tag, Link } from 'lucide-vue-next'

defineProps<{ skill: SkillInfo }>()
const emit = defineEmits<{
  view: [skill: SkillInfo]
  delete: [skill: SkillInfo]
}>()

/** 根据分类返回对应的样式类 */
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

const MAX_TAGS = 3
</script>

<template>
  <article class="group relative flex flex-col bg-surface border border-border rounded-xl p-5 cursor-pointer
           transition-all duration-200
           hover:border-accent/40
           hover:bg-elevated
           hover:shadow-[0_0_0_1px_var(--color-accent),0_6px_28px_oklch(0_0_0/0.15)] dark:hover:shadow-[0_0_0_1px_var(--color-accent),0_6px_28px_oklch(0_0_0/0.4)]" @click="emit('view', skill)">
    <!-- 左侧悬浮指示条 -->
    <div class="absolute left-0 top-4 bottom-4 w-0.75 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-200" style="background: var(--color-accent);" />
    <!-- 顶部行：分类 + 操作按钮 -->
    <div class="flex items-center justify-between mb-3">
      <span class="inline-flex items-center px-2 py-0.5 rounded-md text-xs font-medium border" :class="categoryStyle(skill.category)">
        {{ skill.category || '未分类' }}
      </span>

      <div class="flex items-center gap-0.5">
        <!-- 查看按钮（始终可见，悬浮时高亮） -->
        <button class="p-1.5 rounded-lg text-text-muted opacity-40 group-hover:opacity-100 group-hover:text-text hover:bg-elevated! transition-all duration-150" title="查看详情" @click.stop="emit('view', skill)">
          <Eye :size="14" />
        </button>
        <!-- 删除按钮（悬浮时出现） -->
        <button class="p-1.5 rounded-lg text-text-muted opacity-0 group-hover:opacity-60 hover:opacity-100! hover:text-danger! hover:bg-danger-bg! transition-all duration-150" title="删除技能" @click.stop="emit('delete', skill)">
          <Trash2 :size="14" />
        </button>
      </div>
    </div>

    <!-- 技能名称 -->
    <h3 class="font-display text-base font-semibold text-text leading-snug mb-1.5 truncate" :title="skill.name">
      {{ skill.name }}
    </h3>

    <!-- 描述 -->
    <p class="text-sm text-text-dim leading-relaxed mb-4 flex-1" style="display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; line-clamp: 2;">
      {{ skill.description || '无描述' }}
    </p>

    <!-- 底部：appliesTo 标签 + 依赖数 -->
    <div class="flex flex-wrap items-center gap-1.5 mt-auto">
      <!-- 应用于标签（最多显示 MAX_TAGS 个）-->
      <template v-for="(tag) in skill.appliesTo.slice(0, MAX_TAGS)" :key="tag">
        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-xs
                 bg-input text-text-dim border border-border">
          <Tag :size="10" class="opacity-60" />
          {{ tag }}
        </span>
      </template>

      <!-- 剩余标签数量 -->
      <span v-if="skill.appliesTo.length > MAX_TAGS" class="px-2 py-0.5 rounded-md text-xs bg-input text-text-muted border border-border">
        +{{ skill.appliesTo.length - MAX_TAGS }}
      </span>

      <!-- 无 SKILL.md 警告 -->
      <span v-if="!skill.hasSkillFile" class="ml-auto px-2 py-0.5 rounded-md text-xs bg-warning-bg text-warning border border-warning/30">
        无 SKILL.md
      </span>

      <!-- 依赖数量 -->
      <span v-if="skill.dependsOn.length > 0" class="ml-auto inline-flex items-center gap-1 text-xs text-text-muted" :title="`依赖: ${skill.dependsOn.join(', ')}`">
        <Link :size="10" />
        {{ skill.dependsOn.length }}
      </span>
    </div>
  </article>
</template>
