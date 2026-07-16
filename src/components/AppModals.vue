<script setup lang="ts">
defineProps<{
  showShortcuts: boolean;
  showAbout: boolean;
  contextMenu: { visible: boolean; x: number; y: number; trackIndex: number };
}>();

const emit = defineEmits<{
  closeShortcuts: [];
  closeAbout: [];
  contextMenuAction: [action: string];
  closeContextMenu: [];
}>();
</script>

<template>
  <!-- 右键菜单 -->
  <Teleport to="body">
    <div
      v-if="contextMenu.visible"
      class="context-menu-overlay"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <button class="ctx-item" @click="emit('contextMenuAction', 'play')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M5 3l14 9-14 9V3z" fill="currentColor"/></svg>
        播放
      </button>
      <button class="ctx-item" @click="emit('contextMenuAction', 'favor')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        收藏/取消
      </button>
      <div class="ctx-divider"></div>
      <button class="ctx-item" @click="emit('contextMenuAction', 'reveal')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        在 Finder 中显示
      </button>
      <div class="ctx-divider"></div>
      <button class="ctx-item" @click="emit('contextMenuAction', 'trim')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M6 2v20M18 2v20M6 6l12 12M18 6L6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        裁剪
      </button>
      <button class="ctx-item" @click="emit('contextMenuAction', 'convert')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M14 2v6h6M12 18v-6M9 15l3-3 3 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        格式转换
      </button>
      <div class="ctx-divider"></div>
      <button class="ctx-item danger" @click="emit('contextMenuAction', 'remove')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2m3 0v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6h14z" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>
        移除
      </button>
    </div>
  </Teleport>

  <!-- 快捷键面板 -->
  <Teleport to="body">
    <transition name="fade-scale">
      <div v-if="showShortcuts" class="modal-overlay" @click.self="emit('closeShortcuts')">
        <div class="modal-panel">
          <div class="modal-header">
            <h3>快捷键</h3>
            <button class="modal-close" @click="emit('closeShortcuts')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
            </button>
          </div>
          <div class="shortcuts-grid">
            <div class="shortcut-row"><kbd>Space</kbd><span>播放 / 暂停</span></div>
            <div class="shortcut-row"><kbd>←</kbd><span>上一首</span></div>
            <div class="shortcut-row"><kbd>→</kbd><span>下一首</span></div>
            <div class="shortcut-row"><kbd>S</kbd><span>随机播放</span></div>
            <div class="shortcut-row"><kbd>R</kbd><span>切换循环模式</span></div>
            <div class="shortcut-row"><kbd>F</kbd><span>收藏当前曲目</span></div>
            <div class="shortcut-row"><kbd>O</kbd><span>导入音频</span></div>
            <div class="shortcut-row"><kbd>?</kbd><span>显示 / 隐藏此面板</span></div>
            <div class="shortcut-row"><kbd>右键</kbd><span>曲目操作菜单</span></div>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>

  <!-- 关于面板 -->
  <Teleport to="body">
    <transition name="fade-scale">
      <div v-if="showAbout" class="modal-overlay" @click.self="emit('closeAbout')">
        <div class="about-panel">
          <div class="about-header">
            <div class="about-brand">
              <div class="about-icon">
                <svg width="40" height="40" viewBox="0 0 24 24" fill="none">
                  <rect x="5" y="6" width="2.2" height="12" rx="1.1" fill="currentColor" opacity="0.6"/>
                  <rect x="8.6" y="2" width="2.6" height="20" rx="1.3" fill="currentColor"/>
                  <rect x="12.5" y="4.5" width="3" height="4" rx="1.5" fill="currentColor"/>
                  <rect x="16.8" y="2" width="2.6" height="20" rx="1.3" fill="currentColor"/>
                  <rect x="20.8" y="6" width="2.2" height="12" rx="1.1" fill="currentColor" opacity="0.6"/>
                </svg>
              </div>
              <div>
                <h2>Hanono</h2>
                <p class="about-version">版本 0.4.0</p>
              </div>
            </div>
            <button class="modal-close" @click="emit('closeAbout')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
            </button>
          </div>
          <p class="about-desc">沉浸式本地音乐播放器。支持 FLAC / MP3 / WAV / OGG 等主流格式，拖拽导入、播放列表持久化、系统菜单集成。</p>
          <div class="about-meta">
            <div class="about-meta-item"><span class="meta-label">技术栈</span><span class="meta-value">Tauri 2 + Vue 3 + Rust</span></div>
            <div class="about-meta-item"><span class="meta-label">数据存储</span><span class="meta-value">SQLite (本地)</span></div>
            <div class="about-meta-item"><span class="meta-label">平台支持</span><span class="meta-value">macOS · Windows · Linux</span></div>
          </div>
          <p class="about-copyright">© 2026 Hanono. All rights reserved.</p>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped>
/* 右键菜单 */
.context-menu-overlay {
  position: fixed; z-index: 9999; min-width: 180px;
  background: rgba(18, 18, 36, 0.98);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px; padding: 6px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  display: flex; flex-direction: column; gap: 2px;
}

.ctx-item {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.55rem 0.75rem; border: none; border-radius: 8px;
  background: none; color: #cbd5e1; font-size: 0.82rem;
  font-family: inherit; cursor: pointer; transition: all 0.12s ease;
}
.ctx-item:hover { background: rgba(255, 255, 255, 0.08); color: #f1f5f9; }
.ctx-item.danger { color: #f87171; }
.ctx-item.danger:hover { background: rgba(239, 68, 68, 0.12); }

.ctx-divider { height: 1px; background: rgba(255, 255, 255, 0.08); margin: 3px 6px; }

/* 模态面板 */
.modal-overlay {
  position: fixed; inset: 0; z-index: 9997;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0, 0, 0, 0.5); backdrop-filter: blur(4px);
}

.modal-panel {
  background: rgba(18, 18, 36, 0.98);
  backdrop-filter: blur(24px); -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 16px; padding: 1.5rem; max-width: 380px; width: 90%;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
}

.modal-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 1rem;
}
.modal-header h3 { margin: 0; font-size: 1.1rem; font-weight: 700; color: #f1f5f9; }

.modal-close {
  background: none; border: none; color: #64748b; cursor: pointer;
  padding: 4px; border-radius: 6px; transition: all 0.15s ease;
}
.modal-close:hover { color: #f1f5f9; background: rgba(255,255,255,0.06); }

.shortcuts-grid { display: flex; flex-direction: column; gap: 6px; }

.shortcut-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.45rem 0.6rem; border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
}
.shortcut-row kbd {
  background: rgba(255, 255, 255, 0.08); border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 5px; padding: 2px 8px; font-size: 0.72rem; font-weight: 600;
  color: #94a3b8; font-family: inherit;
}
.shortcut-row span { font-size: 0.82rem; color: #cbd5e1; }

/* 关于面板 */
.about-panel {
  background: rgba(18, 18, 36, 0.98);
  backdrop-filter: blur(24px); -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 16px; padding: 1.5rem;
  max-width: 420px; width: 90%;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
}

.about-header {
  display: flex; align-items: flex-start; justify-content: space-between;
  margin-bottom: 1rem;
}

.about-brand { display: flex; align-items: center; gap: 0.85rem; }

.about-icon {
  width: 48px; height: 48px; border-radius: 14px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  display: flex; align-items: center; justify-content: center; color: #fff;
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.3);
}

.about-header h2 { margin: 0; font-size: 1.25rem; font-weight: 700; color: #f1f5f9; }

.about-version { margin: 0; font-size: 0.8rem; color: #64748b; font-weight: 500; }

.about-desc { font-size: 0.85rem; color: #94a3b8; line-height: 1.6; margin: 0 0 1rem; }

.about-meta { display: flex; flex-direction: column; gap: 8px; margin-bottom: 1rem; }

.about-meta-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 0.45rem 0.6rem; border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
}
.meta-label { font-size: 0.78rem; color: #64748b; font-weight: 500; }
.meta-value { font-size: 0.82rem; color: #e2e8f0; font-weight: 600; }

.about-copyright { font-size: 0.72rem; color: #475569; text-align: center; margin: 0; }

.fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.92); }
</style>
