import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export type ThemeMode = 'light' | 'dark' | 'system';

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<ThemeMode>((localStorage.getItem('theme-mode') as ThemeMode) || 'system');

  const applyTheme = (mode: ThemeMode) => {
    const root = document.documentElement;
    const isDark =
      mode === 'dark' ||
      (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    if (isDark) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }

    // 存储偏好
    localStorage.setItem('theme-mode', mode);
  };

  // 监听系统主题变化
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (theme.value === 'system') {
      applyTheme('system');
    }
  });

  watch(theme, (newTheme) => {
    applyTheme(newTheme);
  }, { immediate: true });

  return {
    theme,
    setTheme(newTheme: ThemeMode) {
      theme.value = newTheme;
    }
  };
});
