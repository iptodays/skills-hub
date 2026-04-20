<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { check } from '@tauri-apps/plugin-updater';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { useThemeStore } from '../stores/theme';
import { Monitor, Sun, Moon } from 'lucide-vue-next';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits(['update:visible']);

const themeStore = useThemeStore();
const version = ref('');
const isAutostartEnabled = ref(false);
const isHidingDockIcon = ref(false);
const updateStatus = ref('');
const isCheckingUpdate = ref(false);

const close = () => {
  emit('update:visible', false);
};

const toggleAutostart = async () => {
  try {
    if (isAutostartEnabled.value) {
      await disable();
      isAutostartEnabled.value = false;
    } else {
      await enable();
      isAutostartEnabled.value = true;
    }
  } catch (e) {
    console.error('设置自启动失败:', e);
  }
};

const toggleDockIcon = async () => {
  try {
    // 这是一个特殊的 Tauri 2.0 command 或者通过 rust 实现
    // 在 MacOS 上隐藏 Dock 图标通常需要修改 activation policy
    // 这里我们先调用后端的一个占位/实现好的 command
    const newState = !isHidingDockIcon.value;
    await invoke('set_dock_icon_visible', { visible: !newState });
    isHidingDockIcon.value = newState;

    // 记忆设置
    localStorage.setItem('hide_dock_icon', newState.toString());
  } catch (e) {
    console.error('设置 Dock 图标失败:', e);
  }
};

const checkForUpdates = async () => {
  if (isCheckingUpdate.value) return;

  isCheckingUpdate.value = true;
  updateStatus.value = '正在检查更新...';

  try {
    const update = await check();
    if (update) {
      updateStatus.value = `发现新版本: ${update.version}`;
      // 调用 downloadAndInstall 下载并安装更新
      await update.downloadAndInstall();
      updateStatus.value = '更新已下载，请重启应用以完成安装';
    } else {
      updateStatus.value = '您当前已是最新版本';
    }
  } catch (e: any) {
    console.error('检查更新失败:', e);
    // 判断是否是网络问题或未配置更新服务器
    if (e?.toString().includes('Update endpoint') || e?.toString().includes('404')) {
      updateStatus.value = '未检测到更新源，请稍后再试';
    } else {
      updateStatus.value = '检查更新时遇到错误，请检查网络连接';
    }
  } finally {
    isCheckingUpdate.value = false;
  }
};

onMounted(async () => {
  version.value = await getVersion();
  isAutostartEnabled.value = await isEnabled();

  const savedHideDock = localStorage.getItem('hide_dock_icon') === 'true';
  isHidingDockIcon.value = savedHideDock;
});
</script>

<template>
  <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="close">
    <div class="w-full max-w-md bg-surface border border-border rounded-xl shadow-2xl overflow-hidden animate-in fade-in zoom-in duration-200">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-border flex items-center justify-between">
        <h3 class="text-lg font-semibold text-text">设置</h3>
        <button @click="close" class="p-1 rounded-lg hover:bg-elevated text-text-muted transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="px-6 py-6 space-y-6">
        <!-- 常规设置 -->
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <div class="text-sm font-medium text-text">开机自启</div>
              <div class="text-xs text-text-muted">在系统启动时自动运行 Skills Hub</div>
            </div>
            <button @click="toggleAutostart" class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent" :class="isAutostartEnabled ? 'bg-accent' : 'bg-input'">
              <span class="inline-block h-4 w-4 transform rounded-full bg-white dark:bg-zinc-100 transition-transform" :class="isAutostartEnabled ? 'translate-x-6' : 'translate-x-1'" />
            </button>
          </div>

          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <div class="text-sm font-medium text-text">隐藏 Dock 图标</div>
              <div class="text-xs text-text-muted">保持应用在后台运行且不占用 Dock 栏</div>
            </div>
            <button @click="toggleDockIcon" class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent" :class="isHidingDockIcon ? 'bg-accent' : 'bg-input'">
              <span class="inline-block h-4 w-4 transform rounded-full bg-white dark:bg-zinc-100 transition-transform" :class="isHidingDockIcon ? 'translate-x-6' : 'translate-x-1'" />
            </button>
          </div>
        </div>

        <div class="h-px bg-border"></div>

        <!-- 主题设置 -->
        <div class="space-y-3">
          <div class="text-sm font-medium text-text">界面主题</div>
          <div class="grid grid-cols-3 gap-2">
            <button @click="themeStore.setTheme('system')" class="flex flex-col items-center gap-1.5 py-2.5 rounded-lg border transition-all"
              :class="themeStore.theme === 'system' ? 'bg-accent/10 border-accent text-accent' : 'bg-transparent border-border text-text-muted hover:bg-elevated'">
              <Monitor :size="18" />
              <span class="text-xs font-medium">跟随系统</span>
            </button>
            <button @click="themeStore.setTheme('light')" class="flex flex-col items-center gap-1.5 py-2.5 rounded-lg border transition-all"
              :class="themeStore.theme === 'light' ? 'bg-accent/10 border-accent text-accent' : 'bg-transparent border-border text-text-muted hover:bg-elevated'">
              <Sun :size="18" />
              <span class="text-xs font-medium">浅色模式</span>
            </button>
            <button @click="themeStore.setTheme('dark')" class="flex flex-col items-center gap-1.5 py-2.5 rounded-lg border transition-all"
              :class="themeStore.theme === 'dark' ? 'bg-accent/10 border-accent text-accent' : 'bg-transparent border-border text-text-muted hover:bg-elevated'">
              <Moon :size="18" />
              <span class="text-xs font-medium">深色模式</span>
            </button>
          </div>
        </div>

        <div class="h-px bg-border"></div>

        <!-- 版本信息 -->
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="text-sm font-medium text-text">版本号</div>
            <div class="text-sm text-text-muted">v{{ version }}</div>
          </div>

          <div class="space-y-2">
            <button @click="checkForUpdates" :disabled="isCheckingUpdate" class="w-full py-2 px-4 bg-elevated hover:bg-input disabled:opacity-50 text-text text-sm font-medium rounded-lg transition-colors flex items-center justify-center space-x-2">
              <svg v-if="isCheckingUpdate" class="animate-spin" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12a9 9 0 1 1-6.219-8.56" />
              </svg>
              <span>检查更新</span>
            </button>
            <p v-if="updateStatus" class="text-center text-xs text-text-muted">{{ updateStatus }}</p>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-elevated/30 border-t border-border flex justify-end">
        <button @click="close" class="px-4 py-2 bg-accent hover:opacity-90 text-white font-medium rounded-lg transition-colors">
          完成
        </button>
      </div>
    </div>
  </div>
</template>
