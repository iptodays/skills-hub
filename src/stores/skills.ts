import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SkillInfo, InstallResult } from '../types/skill'

export const useSkillsStore = defineStore('skills', () => {
  const skills = ref<SkillInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const searchQuery = ref('')
  const selectedCategory = ref('全部')
  const skillsDir = ref('')

  /** 去重分类列表 */
  const categories = computed(() => {
    const cats = new Set(
      skills.value
        .map(s => s.category || '未分类')
        .filter(Boolean)
    )
    return ['全部', ...Array.from(cats).sort()]
  })

  /** 分类计数 */
  const categoryCount = computed(() => {
    const counts: Record<string, number> = { '全部': skills.value.length }
    skills.value.forEach(s => {
      const cat = s.category || '未分类'
      counts[cat] = (counts[cat] ?? 0) + 1
    })
    return counts
  })

  /** 搜索 + 分类过滤后的技能列表 */
  const filteredSkills = computed(() => {
    const query = searchQuery.value.toLowerCase().trim()
    return skills.value.filter(skill => {
      const matchesSearch =
        !query ||
        skill.name.toLowerCase().includes(query) ||
        skill.description.toLowerCase().includes(query) ||
        skill.category.toLowerCase().includes(query) ||
        skill.appliesTo.some(t => t.toLowerCase().includes(query)) ||
        skill.dependsOn.some(t => t.toLowerCase().includes(query))

      const matchesCategory =
        selectedCategory.value === '全部' ||
        (skill.category || '未分类') === selectedCategory.value

      return matchesSearch && matchesCategory
    })
  })

  /** 从 Rust 后端加载所有 skills */
  async function fetchSkills() {
    loading.value = true
    error.value = null
    try {
      // Rust 返回 snake_case，需要手动转换
      const raw = await invoke<Array<Record<string, unknown>>>('list_skills')
      skills.value = raw.map(r => ({
        name: String(r.name ?? ''),
        description: String(r.description ?? ''),
        category: String(r.category ?? ''),
        dirName: String(r.dir_name ?? ''),
        appliesTo: (r.applies_to as string[]) ?? [],
        dependsOn: (r.depends_on as string[]) ?? [],
        hasSkillFile: Boolean(r.has_skill_file),
      }))
      skillsDir.value = await invoke<string>('get_skills_dir_path')
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** 读取 skill 的 Markdown 内容 */
  async function getSkillContent(dirName: string): Promise<string> {
    // Tauri v2 将 Rust snake_case 参数自动映射为 camelCase
    return await invoke<string>('get_skill_content', { dirName })
  }

  /** 通过 git URL 安装 skill */
  async function installSkillGit(url: string): Promise<InstallResult> {
    const raw = await invoke<Record<string, unknown>>('install_skill_git', { url })
    return {
      success: Boolean(raw.success),
      dirName: String(raw.dir_name ?? ''),
      message: String(raw.message ?? ''),
    }
  }

  /** 删除指定 skill */
  async function deleteSkill(dirName: string): Promise<void> {
    await invoke('delete_skill', { dirName })
    skills.value = skills.value.filter(s => s.dirName !== dirName)
  }

  return {
    skills,
    loading,
    error,
    searchQuery,
    selectedCategory,
    skillsDir,
    categories,
    categoryCount,
    filteredSkills,
    fetchSkills,
    getSkillContent,
    installSkillGit,
    deleteSkill,
  }
})
