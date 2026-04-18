/** Skill 核心元数据，来自 SKILL.md frontmatter */
export interface SkillInfo {
  name: string
  description: string
  category: string
  dirName: string
  appliesTo: string[]
  dependsOn: string[]
  hasSkillFile: boolean
}

/** git 安装结果 */
export interface InstallResult {
  success: boolean
  dirName: string
  message: string
}
