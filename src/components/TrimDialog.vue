<script setup lang="ts">
import { ref, watch, computed } from 'vue';

const props = defineProps<{
  show: boolean;
  trackName: string;
  duration: number;
}>();

const emit = defineEmits<{
  close: [];
  trim: [startSec: number, endSec: number];
}>();

const startMin = ref(0);
const startSec = ref(0);
const endMin = ref(0);
const endSec = ref(30);

// Reset on open
watch(() => props.show, (v) => {
  if (v && props.duration > 0) {
    const total = props.duration;
    endMin.value = Math.min(Math.floor(total / 60), 99);
    endSec.value = Math.min(Math.floor(total % 60), total < 30 ? total : 30);
    startMin.value = 0;
    startSec.value = 0;
  }
});

function getSeconds(min: number, sec: number) {
  return min * 60 + sec;
}

function onConfirm() {
  const s = getSeconds(startMin.value, startSec.value);
  const e = getSeconds(endMin.value, endSec.value);
  if (e <= s) return;
  emit('trim', s, e);
}

function clampTime(min: number, sec: number): { min: number; sec: number } {
  let total = Math.max(0, Math.min(props.duration, min * 60 + sec));
  return { min: Math.floor(total / 60), sec: Math.floor(total % 60) };
}

function onStartMinInput(e: Event) {
  const v = parseInt((e.target as HTMLInputElement).value) || 0;
  const r = clampTime(v, startSec.value);
  startMin.value = r.min; startSec.value = r.sec;
}
function onStartSecInput(e: Event) {
  const v = parseInt((e.target as HTMLInputElement).value) || 0;
  const r = clampTime(startMin.value, v);
  startMin.value = r.min; startSec.value = r.sec;
}
function onEndMinInput(e: Event) {
  const v = parseInt((e.target as HTMLInputElement).value) || 0;
  const r = clampTime(v, endSec.value);
  endMin.value = r.min; endSec.value = r.sec;
}
function onEndSecInput(e: Event) {
  const v = parseInt((e.target as HTMLInputElement).value) || 0;
  const r = clampTime(endMin.value, v);
  endMin.value = r.min; endSec.value = r.sec;
}

const previewDuration = computed(() => {
  const diff = getSeconds(endMin.value, endSec.value) - getSeconds(startMin.value, startSec.value);
  return Math.max(0, diff);
});
</script>

<template>
  <Teleport to="body">
    <transition name="fade-scale">
      <div v-if="show" class="trim-overlay" @click.self="emit('close')">
        <div class="trim-panel">
          <div class="trim-header">
            <h3>✂️ 音频裁剪</h3>
            <button class="trim-close" @click="emit('close')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
            </button>
          </div>

          <p class="trim-filename">{{ trackName }}</p>

          <div class="trim-time-row">
            <div class="time-group">
              <label class="time-label">开始</label>
              <div class="time-inputs">
                <input class="time-inp" type="number" min="0" :max="99"
                  :value="startMin" @input="onStartMinInput" /> <span class="time-colon">:</span>
                <input class="time-inp" type="number" min="0" max="59"
                  :value="startSec" @input="onStartSecInput" />
              </div>
            </div>
            <span class="time-arrow">→</span>
            <div class="time-group">
              <label class="time-label">结束</label>
              <div class="time-inputs">
                <input class="time-inp" type="number" min="0" :max="99"
                  :value="endMin" @input="onEndMinInput" /> <span class="time-colon">:</span>
                <input class="time-inp" type="number" min="0" max="59"
                  :value="endSec" @input="onEndSecInput" />
              </div>
            </div>
          </div>

          <p class="trim-preview">裁剪后时长 ≈ {{ previewDuration }} 秒</p>

          <div class="trim-actions">
            <button class="trim-btn cancel" @click="emit('close')">取消</button>
            <button class="trim-btn confirm" @click="onConfirm" :disabled="previewDuration <= 0">开始裁剪</button>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped>
.trim-overlay {
  position: fixed; inset: 0; z-index: 9999;
  display: flex; align-items: center; justify-content: center;
  background: rgba(0, 0, 0, 0.5); backdrop-filter: blur(4px);
}

.trim-panel {
  background: rgba(18, 18, 36, 0.98);
  backdrop-filter: blur(24px); -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 16px; padding: 1.5rem;
  max-width: 380px; width: 92%;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
}

.trim-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 0.5rem;
}
.trim-header h3 { margin: 0; font-size: 1.1rem; font-weight: 700; color: #f1f5f9; }

.trim-close {
  background: none; border: none; color: #64748b; cursor: pointer;
  padding: 4px; border-radius: 6px; transition: all 0.15s ease;
}
.trim-close:hover { color: #f1f5f9; background: rgba(255,255,255,0.06); }

.trim-filename {
  font-size: 0.78rem; color: #64748b; margin: 0 0 1.2rem;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

.trim-time-row {
  display: flex; align-items: flex-end; justify-content: center; gap: 1rem;
  margin-bottom: 0.8rem;
}

.time-group { display: flex; flex-direction: column; align-items: center; gap: 6px; }
.time-label { font-size: 0.75rem; color: #94a3b8; font-weight: 500; }

.time-inputs {
  display: flex; align-items: center; gap: 4px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px; padding: 6px 10px;
}

.time-inp {
  width: 44px; text-align: center;
  background: transparent; border: none; outline: none;
  color: #f1f5f9; font-size: 1.1rem; font-weight: 700;
  font-family: inherit; font-variant-numeric: tabular-nums;
  -moz-appearance: textfield;
}
.time-inp::-webkit-outer-spin-button,
.time-inp::-webkit-inner-spin-button { -webkit-appearance: none; margin: 0; }

.time-colon {
  font-size: 1.1rem; font-weight: 700; color: #64748b;
}

.time-arrow {
  font-size: 1.2rem; color: #64748b; padding-bottom: 8px;
}

.trim-preview {
  text-align: center; font-size: 0.8rem; color: #a78bfa;
  margin: 0 0 1rem; font-weight: 500;
}

.trim-actions {
  display: flex; justify-content: flex-end; gap: 0.5rem;
}

.trim-btn {
  padding: 0.6rem 1.2rem; border-radius: 10px; font-size: 0.85rem;
  font-weight: 600; cursor: pointer; font-family: inherit; transition: all 0.15s ease;
  border: none;
}
.trim-btn.cancel {
  background: rgba(255, 255, 255, 0.06); color: #94a3b8;
}
.trim-btn.cancel:hover { background: rgba(255, 255, 255, 0.1); color: #f1f5f9; }
.trim-btn.confirm {
  background: linear-gradient(135deg, #6366f1, #8b5cf6); color: #fff;
  box-shadow: 0 4px 20px rgba(99, 102, 241, 0.25);
}
.trim-btn.confirm:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 24px rgba(99, 102, 241, 0.35);
}
.trim-btn.confirm:disabled { opacity: 0.4; cursor: not-allowed; }

.fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.92); }
</style>
