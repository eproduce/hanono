<script setup lang="ts">
import type { EqPresetKey } from '../composables/useAudioEngine';

interface EqBand {
  name: string;
  freq: number;
  gain: { value: number };
  q: number;
}

defineProps<{
  show: boolean;
  eqBands: EqBand[];
  bassBoost: number;
  surroundAmount: number;
  reverbAmount: number;
  currentPreset: EqPresetKey;
  eqPresets: Record<EqPresetKey, number[]>;
}>();

const emit = defineEmits<{
  close: [];
  applyPreset: [key: EqPresetKey];
  onBassBoostInput: [e: Event];
  onSurroundInput: [e: Event];
  onReverbInput: [e: Event];
  onEqBandInput: [index: number, e: Event];
  resetAll: [];
}>();

const presetDisplay: Record<EqPresetKey, string> = {
  flat: '默认', pop: '流行', rock: '摇滚', jazz: '爵士',
  classical: '古典', vocal: '人声', bass: '低音',
};
</script>

<template>
  <Teleport to="body">
    <transition name="fade-scale">
      <div v-if="show" class="fx-overlay" @click.self="emit('close')">
        <div class="fx-panel">
          <div class="fx-header">
            <h3>🎛️ 音效增强</h3>
            <button class="fx-close" @click="emit('close')">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
            </button>
          </div>

          <div class="fx-section">
            <label class="fx-label">预设</label>
            <div class="preset-grid">
              <button v-for="(_g, key) in eqPresets" :key="key"
                class="preset-btn" :class="{ active: currentPreset === key }"
                @click="emit('applyPreset', key as EqPresetKey)"
              >{{ presetDisplay[key as EqPresetKey] }}</button>
            </div>
          </div>

          <div class="fx-section">
            <label class="fx-label">均衡器</label>
            <div class="eq-sliders">
              <div v-for="(band, i) in eqBands" :key="band.name" class="eq-band">
                <input class="eq-slider" type="range" min="-12" max="12" step="0.5"
                  :value="band.gain.value" @input="emit('onEqBandInput', i, $event)" />
                <span class="eq-db">{{ (band.gain.value ?? 0) > 0 ? '+' : '' }}{{ (band.gain.value ?? 0).toFixed(1) }}dB</span>
                <span class="eq-freq">{{ band.name }}</span>
              </div>
            </div>
          </div>

          <div class="fx-section">
            <label class="fx-label">
              低音增强
              <span class="fx-val">{{ bassBoost > 0 ? '+' + bassBoost.toFixed(1) + 'dB' : '关闭' }}</span>
            </label>
            <input class="fx-range" type="range" min="0" max="15" step="0.5"
              :value="bassBoost" @input="emit('onBassBoostInput', $event)" />
          </div>

          <div class="fx-section">
            <label class="fx-label">
              3D环绕
              <span class="fx-val">{{ surroundAmount > 0 ? (surroundAmount * 100).toFixed(0) + '%' : '关闭' }}</span>
            </label>
            <input class="fx-range" type="range" min="0" max="1" step="0.05"
              :value="surroundAmount" @input="emit('onSurroundInput', $event)" />
          </div>

          <div class="fx-section">
            <label class="fx-label">
              混响
              <span class="fx-val">{{ reverbAmount > 0 ? (reverbAmount * 100).toFixed(0) + '%' : '关闭' }}</span>
            </label>
            <input class="fx-range" type="range" min="0" max="1" step="0.05"
              :value="reverbAmount" @input="emit('onReverbInput', $event)" />
          </div>

          <div class="fx-reset-row">
            <button class="fx-ghost-btn" @click="emit('resetAll')">重置全部</button>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped>
.fx-overlay {
  position: fixed;
  inset: 0;
  z-index: 9998;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.fx-panel {
  background: rgba(18, 18, 36, 0.98);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 16px;
  padding: 1.5rem;
  max-width: 480px;
  width: 92%;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  max-height: 85vh;
  overflow-y: auto;
}

.fx-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.fx-header h3 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
  color: #f1f5f9;
}

.fx-close {
  background: none;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: all 0.15s ease;
}
.fx-close:hover { color: #f1f5f9; background: rgba(255,255,255,0.06); }

.fx-section { margin-bottom: 1rem; }

.fx-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.85rem;
  font-weight: 600;
  color: #e2e8f0;
  margin-bottom: 0.5rem;
}

.fx-val { font-weight: 500; font-size: 0.78rem; color: #a78bfa; }

.fx-range {
  -webkit-appearance: none; appearance: none;
  width: 100%; height: 6px; border-radius: 999px;
  background: rgba(255, 255, 255, 0.08); cursor: pointer; outline: none;
}
.fx-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px; height: 18px; border-radius: 50%;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border: 2px solid #fff; cursor: pointer;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}

.preset-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 0.4rem; }

.preset-btn {
  padding: 0.45rem 0.65rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
  color: #94a3b8;
  font-size: 0.78rem; font-weight: 500;
  cursor: pointer; font-family: inherit;
  transition: all 0.18s ease; white-space: nowrap;
}
.preset-btn:hover { background: rgba(99, 102, 241, 0.1); color: #e2e8f0; border-color: rgba(99, 102, 241, 0.25); }
.preset-btn.active {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.25), rgba(139, 92, 246, 0.2));
  color: #a78bfa; border-color: #6366f1; font-weight: 600;
}

.eq-sliders { display: flex; gap: 0.5rem; justify-content: center; }
.eq-band { display: flex; flex-direction: column; align-items: center; gap: 0.3rem; flex: 1; max-width: 64px; }

.eq-slider {
  -webkit-appearance: none; appearance: none;
  width: 100%; height: 6px; cursor: pointer;
  background: rgba(255, 255, 255, 0.08); border-radius: 999px;
  outline: none; margin-bottom: 0.3rem;
}
.eq-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 20px; height: 20px; border-radius: 50%;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border: 2px solid #fff; cursor: pointer;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}

.eq-db { font-size: 0.65rem; font-weight: 600; color: #a78bfa; font-variant-numeric: tabular-nums; }
.eq-freq { font-size: 0.68rem; color: #64748b; font-weight: 500; }

.fx-reset-row { display: flex; justify-content: flex-end; margin-top: 0.75rem; }

.fx-ghost-btn {
  border: none; padding: 0.6rem 1rem; border-radius: 12px; cursor: pointer;
  font-weight: 600; font-size: 0.85rem; font-family: inherit;
  background: rgba(255, 255, 255, 0.06); color: #94a3b8;
  border: 1px solid rgba(255, 255, 255, 0.08);
  transition: all 0.2s ease;
}
.fx-ghost-btn:hover { background: rgba(255, 255, 255, 0.1); color: #f1f5f9; }

.fade-scale-enter-active, .fade-scale-leave-active { transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.fade-scale-enter-from, .fade-scale-leave-to { opacity: 0; transform: scale(0.92); }
</style>
