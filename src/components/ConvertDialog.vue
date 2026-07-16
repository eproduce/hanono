<script setup lang="ts">
import { ref } from 'vue';

defineProps<{
  show: boolean;
  trackName: string;
}>();

const emit = defineEmits<{
  close: [];
  convert: [format: string, bitrate: string];
}>();

const formats = [
  { id: 'mp3',   label: 'MP3',     desc: '通用，有损压缩',   bitrates: ['128k', '192k', '256k', '320k'], icon: '🎵' },
  { id: 'flac',  label: 'FLAC',    desc: '无损压缩，品质最高', bitrates: [] as string[],              icon: '💎' },
  { id: 'aac',   label: 'AAC/M4A', desc: '高效有损，Apple 生态', bitrates: ['128k', '192k', '256k'], icon: '🍎' },
  { id: 'ogg',   label: 'OGG',     desc: '开源有损，高压缩率',   bitrates: ['3', '5', '7'],          icon: '🔊' },
  { id: 'wav',   label: 'WAV',     desc: '无损无压缩，体积大',   bitrates: [] as string[],              icon: '📀' },
];

const selectedFormat = ref('mp3');
const selectedBitrate = ref('320k');

function onConfirm() {
  emit('convert', selectedFormat.value, selectedBitrate.value);
}
</script>

<template>
  <Teleport to="body">
    <transition name="fade-scale">
      <div v-if="show" class="convert-overlay" @click.self="emit('close')">
        <div class="convert-panel">
          <div class="convert-header">
            <h3>🔄 格式转换</h3>
            <button class="convert-close" @click="emit('close')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
            </button>
          </div>

          <p class="convert-filename">{{ trackName }}</p>

          <div class="format-list">
            <button
              v-for="fmt in formats" :key="fmt.id"
              class="format-option"
              :class="{ active: selectedFormat === fmt.id }"
              @click="selectedFormat = fmt.id; selectedBitrate = fmt.bitrates[fmt.bitrates.length - 1] || ''"
            >
              <span class="format-icon">{{ fmt.icon }}</span>
              <span class="format-info">
                <span class="format-label">{{ fmt.label }}</span>
                <span class="format-desc">{{ fmt.desc }}</span>
              </span>
            </button>
          </div>

          <div v-if="formats.find(f => f.id === selectedFormat)?.bitrates.length" class="bitrate-row">
            <span class="bitrate-label">品质</span>
            <div class="bitrate-options">
              <button
                v-for="br in formats.find(f => f.id === selectedFormat)!.bitrates"
                :key="br"
                class="bitrate-btn"
                :class="{ active: selectedBitrate === br }"
                @click="selectedBitrate = br"
              >{{ br }}</button>
            </div>
          </div>

          <div class="convert-actions">
            <button class="convert-btn cancel" @click="emit('close')">取消</button>
            <button class="convert-btn confirm" @click="onConfirm">开始转换</button>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped>
.convert-overlay {
  position: fixed; inset: 0; z-index: 9999;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0, 0, 0, 0.5); backdrop-filter: blur(4px);
}

.convert-panel {
  background: rgba(18, 18, 36, 0.98);
  backdrop-filter: blur(24px); -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 16px; padding: 1.5rem;
  max-width: 400px; width: 92%;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
}

.convert-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 0.5rem;
}
.convert-header h3 { margin: 0; font-size: 1.1rem; font-weight: 700; color: #f1f5f9; }

.convert-close {
  background: none; border: none; color: #64748b; cursor: pointer;
  padding: 4px; border-radius: 6px; transition: all 0.15s ease;
}
.convert-close:hover { color: #f1f5f9; background: rgba(255,255,255,0.06); }

.convert-filename {
  font-size: 0.78rem; color: #64748b; margin: 0 0 1rem;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

.format-list { display: flex; flex-direction: column; gap: 6px; margin-bottom: 1rem; }

.format-option {
  display: flex; align-items: center; gap: 0.75rem;
  padding: 0.65rem 0.85rem; border-radius: 10px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  cursor: pointer; font-family: inherit;
  transition: all 0.15s ease; text-align: left;
}
.format-option:hover { background: rgba(99, 102, 241, 0.08); border-color: rgba(99, 102, 241, 0.2); }
.format-option.active {
  background: rgba(99, 102, 241, 0.12);
  border-color: rgba(99, 102, 241, 0.3);
}

.format-icon { font-size: 1.4rem; flex-shrink: 0; width: 32px; text-align: center; }
.format-info { display: flex; flex-direction: column; gap: 2px; }
.format-label { font-size: 0.88rem; font-weight: 600; color: #e2e8f0; }
.format-desc { font-size: 0.7rem; color: #64748b; }

.bitrate-row {
  display: flex; align-items: center; gap: 0.75rem;
  padding: 0.5rem 0.85rem; border-radius: 10px;
  background: rgba(255, 255, 255, 0.02);
  margin-bottom: 1rem;
}

.bitrate-label { font-size: 0.78rem; font-weight: 500; color: #94a3b8; flex-shrink: 0; }
.bitrate-options { display: flex; gap: 6px; }
.bitrate-btn {
  padding: 4px 12px; border-radius: 6px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  color: #94a3b8; font-size: 0.78rem; font-weight: 600;
  cursor: pointer; font-family: inherit; transition: all 0.15s ease;
}
.bitrate-btn:hover { color: #e2e8f0; border-color: rgba(255, 255, 255, 0.15); }
.bitrate-btn.active {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
  color: #a78bfa;
}

.convert-actions {
  display: flex; justify-content: flex-end; gap: 0.5rem;
}

.convert-btn {
  padding: 0.6rem 1.2rem; border-radius: 10px; font-size: 0.85rem;
  font-weight: 600; cursor: pointer; font-family: inherit; transition: all 0.15s ease;
  border: none;
}
.convert-btn.cancel {
  background: rgba(255, 255, 255, 0.06); color: #94a3b8;
}
.convert-btn.cancel:hover { background: rgba(255, 255, 255, 0.1); color: #f1f5f9; }
.convert-btn.confirm {
  background: linear-gradient(135deg, #6366f1, #8b5cf6); color: #fff;
  box-shadow: 0 4px 20px rgba(99, 102, 241, 0.25);
}
.convert-btn.confirm:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 24px rgba(99, 102, 241, 0.35);
}

.fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.92); }
</style>
