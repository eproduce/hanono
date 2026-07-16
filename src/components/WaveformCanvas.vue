<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue';

const props = defineProps<{
  peaks: number[];
  currentTime: number;
  duration: number;
  loading?: boolean;
}>();

const emit = defineEmits<{
  seek: [time: number];
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);
const hoverTime = ref<number | null>(null);
const hoverX = ref(0);
const isDragging = ref(false);

// ResizeObserver to handle canvas resize
let resizeObserver: ResizeObserver | null = null;

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  ctx.scale(dpr, dpr);

  const w = rect.width;
  const h = rect.height;
  const peaks = props.peaks;

  ctx.clearRect(0, 0, w, h);

  if (peaks.length === 0) {
    // Draw flat line when no data
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(0, h / 2);
    ctx.lineTo(w, h / 2);
    ctx.stroke();
    return;
  }

  const progress = props.duration > 0 ? props.currentTime / props.duration : 0;
  const barWidth = w / peaks.length;
  const midY = h / 2;

  for (let i = 0; i < peaks.length; i++) {
    const x = i * barWidth;
    const peakH = Math.max(peaks[i] * midY * 0.92, 1.0);
    const isPast = i / peaks.length <= progress;

    // Bar gradient
    if (isPast) {
      const grad = ctx.createLinearGradient(x, midY - peakH, x, midY + peakH);
      grad.addColorStop(0, 'rgba(99, 102, 241, 0.9)');
      grad.addColorStop(0.5, 'rgba(139, 92, 246, 0.8)');
      grad.addColorStop(1, 'rgba(99, 102, 241, 0.9)');
      ctx.fillStyle = grad;
    } else {
      ctx.fillStyle = 'rgba(255, 255, 255, 0.15)';
    }

    const barW = Math.max(barWidth * 0.7, 1.0);
    const xCentered = x + (barWidth - barW) / 2;
    ctx.fillRect(xCentered, midY - peakH, barW, peakH * 2);
  }

  // Hover indicator
  if (hoverTime.value !== null && !isDragging.value && props.duration > 0) {
    const hx = hoverX.value;
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    ctx.beginPath();
    ctx.moveTo(hx, 0);
    ctx.lineTo(hx, h);
    ctx.stroke();
    ctx.setLineDash([]);

    // Time tooltip bar
    const mins = Math.floor(hoverTime.value / 60);
    const secs = Math.floor(hoverTime.value % 60).toString().padStart(2, '0');
    const label = `${mins}:${secs}`;
    ctx.font = '11px "Inter", "SF Pro Display", sans-serif';
    const textW = ctx.measureText(label).width;
    const tx = Math.max(4, Math.min(w - textW - 10, hx - textW / 2));
    const ty = 16;
    ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
    ctx.beginPath();
    ctx.roundRect(tx - 4, ty - 12, textW + 8, 18, 4);
    ctx.fill();
    ctx.fillStyle = '#fff';
    ctx.fillText(label, tx, ty);
  }
}

function getTimeFromX(x: number): number {
  if (!props.duration) return 0;
  const canvas = canvasRef.value;
  if (!canvas) return 0;
  const w = canvas.getBoundingClientRect().width;
  return Math.max(0, Math.min(props.duration, (x / w) * props.duration));
}

function onMouseMove(e: MouseEvent) {
  const rect = canvasRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  hoverX.value = x;
  hoverTime.value = getTimeFromX(x);
  if (isDragging.value) {
    emit('seek', hoverTime.value);
  }
}

function onMouseLeave() {
  if (!isDragging.value) {
    hoverTime.value = null;
  }
}

function onMouseDown(e: MouseEvent) {
  const rect = canvasRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left;
  isDragging.value = true;
  const t = getTimeFromX(x);
  emit('seek', t);
}

function onMouseUp() {
  isDragging.value = false;
}

onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      nextTick(draw);
    });
    resizeObserver.observe(containerRef.value);
  }
  nextTick(draw);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  window.removeEventListener('mouseup', onMouseUp);
});

watch(() => [props.peaks, props.currentTime, props.duration, props.loading], () => {
  nextTick(draw);
}, { deep: false });

// Global mouseup to catch release outside canvas
if (typeof window !== 'undefined') {
  window.addEventListener('mouseup', onMouseUp);
}
</script>

<template>
  <div
    ref="containerRef"
    class="waveform-container"
    :class="{ loading: loading, empty: peaks.length === 0 && !loading }"
  >
    <!-- Loading skeleton -->
    <div v-if="loading" class="waveform-loading">
      <div class="loading-bar" v-for="i in 40" :key="i" :style="{ animationDelay: (i * 0.03) + 's', height: (20 + Math.sin(i * 0.7) * 60) + '%' }"></div>
    </div>
    <!-- Empty placeholder -->
    <div v-else-if="peaks.length === 0" class="waveform-empty">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
        <path d="M3 12h3l2-8h2l4 16h2l2-8h3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <span>暂无波形数据</span>
    </div>
    <!-- Canvas -->
    <canvas
      ref="canvasRef"
      class="waveform-canvas"
      @mousemove="onMouseMove"
      @mouseleave="onMouseLeave"
      @mousedown="onMouseDown"
    ></canvas>
  </div>
</template>

<style scoped>
.waveform-container {
  position: relative;
  width: 100%;
  height: 68px;
  border-radius: 12px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  transition: border-color 0.3s ease;
  cursor: pointer;
}

.waveform-container:hover {
  border-color: rgba(255, 255, 255, 0.12);
}

.waveform-container.loading {
  cursor: default;
}

.waveform-canvas {
  width: 100%;
  height: 100%;
  display: block;
}

.waveform-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 2px;
  padding: 0 12px;
}

.loading-bar {
  flex: 1;
  min-width: 2px;
  background: rgba(99, 102, 241, 0.3);
  border-radius: 2px;
  animation: loading-pulse 0.8s ease-in-out infinite alternate;
}

@keyframes loading-pulse {
  from { opacity: 0.3; }
  to { opacity: 0.8; }
}

.waveform-empty {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: rgba(255, 255, 255, 0.2);
  font-size: 0.8rem;
}
</style>
