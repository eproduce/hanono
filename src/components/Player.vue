<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { invoke, isTauri, convertFileSrc } from '@tauri-apps/api/core';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import WaveformCanvas from './WaveformCanvas.vue';
import FxPanel from './FxPanel.vue';
import PlaylistSidebar from './PlaylistSidebar.vue';
import AppModals from './AppModals.vue';
import ConvertDialog from './ConvertDialog.vue';
import TrimDialog from './TrimDialog.vue';
import { useAudioEngine } from '../composables/useAudioEngine';
import type { EqPresetKey } from '../composables/useAudioEngine';

interface Track {
  name: string;
  url: string;
  path?: string;
  persistent?: boolean;
}

const audio = new Audio();

// ========== 音效系统 (composable) ==========
const {
  eqBands, bassBoost, surroundAmount, reverbAmount,
  currentPreset, currentFxLabel, showFxPanel, eqPresets,
  ensureAudioContext, applyPreset,
  onBassBoostInput, onSurroundInput, onReverbInput, onEqBandInput,
  resetAllEffects, setMasterVolume, openFxPanel,
} = useAudioEngine(audio);
const playlist = ref<Track[]>([]);
const favorites = ref<Track[]>([]);

// 播放历史
interface HistoryEntry { name: string; time: number; }
const playHistory = ref<HistoryEntry[]>([]);
const MAX_HISTORY = 100;

function recordPlay(track: Track) {
  const entry: HistoryEntry = { name: track.name, time: Date.now() };
  // 去重：移除同名记录
  const idx = playHistory.value.findIndex(h => h.name === entry.name);
  if (idx >= 0) playHistory.value.splice(idx, 1);
  playHistory.value.unshift(entry);
  if (playHistory.value.length > MAX_HISTORY) playHistory.value.pop();
}

const currentIndex = ref(-1);
const isPlaying = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const volume = ref(1);

function setVolume(v: number) {
  volume.value = v;
  audio.volume = v;
  const loudGain = (audioInfo.value as any)?._loudnessGain ?? 1.0;
  setMasterVolume(v * loudGain);
  if (v > 0) localStorage.setItem('hanono_prev_volume', String(v));
}

function onVolumeInput(e: Event) {
  setVolume(parseFloat((e.target as HTMLInputElement).value));
}

function toggleMute() {
  if (volume.value > 0) {
    audio.volume = 0;
    setMasterVolume(0);
    volume.value = 0;
  } else {
    const prev = parseFloat(localStorage.getItem('hanono_prev_volume') || '0.7');
    audio.volume = prev;
    const loudGain = (audioInfo.value as any)?._loudnessGain ?? 1.0;
    setMasterVolume(prev * loudGain);
    volume.value = prev;
  }
}
const isSeeking = ref(false);
const seekPreview = ref(0);
const shuffle = ref(false);
const repeatMode = ref<'off' | 'one' | 'all'>('off');
const fileInputRef = ref<HTMLInputElement | null>(null);
const isRestoring = ref(false); // blocks saveState during initial state restore

// 右键菜单
const contextMenu = ref({ visible: false, x: 0, y: 0, trackIndex: -1 });
// 显示快捷键面板
const showShortcuts = ref(false);
// 显示关于面板
const showAbout = ref(false);
// 格式转换
const showConvertDialog = ref(false);
const convertingTrackName = ref('');
const convertingTrackPath = ref('');

// 裁剪
const showTrimDialog = ref(false);
const trimTrackName = ref('');
const trimTrackPath = ref('');

// 封面动效模式: 'none' | 'rotate' | 'pulse' | 'glow'
const coverEffect = ref<'none' | 'rotate' | 'pulse' | 'glow'>('glow');
const effectLabels: Record<string, string> = {
  none: '无动效',
  rotate: '旋转唱片',
  pulse: '呼吸灯',
  glow: '流光光晕',
};

function cycleCoverEffect() {
  const modes: Array<'none' | 'rotate' | 'pulse' | 'glow'> = ['none', 'rotate', 'pulse', 'glow'];
  const idx = modes.indexOf(coverEffect.value);
  coverEffect.value = modes[(idx + 1) % modes.length];
}

// Toast 通知系统
interface Toast { id: number; message: string; type: 'info' | 'error' | 'success'; }
const toasts = ref<Toast[]>([]);
let toastId = 0;

function showToast(message: string, type: 'info' | 'error' | 'success' = 'info') {
  const id = ++toastId;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    const idx = toasts.value.findIndex(t => t.id === id);
    if (idx >= 0) toasts.value.splice(idx, 1);
  }, 3000);
}

// 睡眠定时器
const sleepMinutes = ref(0);
let sleepTimer: ReturnType<typeof setTimeout> | null = null;

// 播放速度
const playbackRate = ref(1);
const speeds = [0.5, 0.75, 1, 1.25, 1.5, 2];

function cycleSpeed() {
  const idx = speeds.indexOf(playbackRate.value);
  playbackRate.value = speeds[(idx + 1) % speeds.length];
  audio.playbackRate = playbackRate.value;
}

// Mini 播放器模式
const isMini = ref(false);

function toggleMini() {
  isMini.value = !isMini.value;
}

// 歌词系统
interface LyricLine { time: number; text: string; }
const lyrics = ref<LyricLine[]>([]);
const currentLyricIdx = ref(-1);

// 波形数据
const waveformPeaks = ref<number[]>([]);
const waveformLoading = ref(false);
let waveformAbortId = 0;
let infoAbortId = 0;

// 音频详情
interface AudioInfo {
  codec: string;
  sampleRate: number;
  sampleRateStr: string;
  channels: number;
  channelsStr: string;
  bitDepth: number;
  bitDepthStr: string;
  bitrate: number;
  bitrateStr: string;
  durationSecs: number;
  durationStr: string;
  fileSize: number;
  fileSizeStr: string;
}
const audioInfo = ref<AudioInfo | null>(null);

function parseLrc(lrcText: string): LyricLine[] {
  const lines: LyricLine[] = [];
  const regex = /\[(\d{2}):(\d{2}(?:\.\d{2,3})?)\](.*)/;
  for (const line of lrcText.split('\n')) {
    const m = line.match(regex);
    if (m) {
      const min = parseInt(m[1]);
      const sec = parseFloat(m[2]);
      const text = m[3].trim();
      if (text) lines.push({ time: min * 60 + sec, text });
    }
  }
  return lines.sort((a, b) => a.time - b.time);
}

async function loadLyrics(track: Track) {
  lyrics.value = [];
  currentLyricIdx.value = -1;
  if (!track.path || !isTauri()) return;
  const lrcPath = track.path.replace(/\.[^.]+$/, '.lrc');
  try {
    const text = await invoke<string>('read_text_file', { path: lrcPath });
    lyrics.value = parseLrc(text);
  } catch { /* 无歌词文件 */ }
}

function updateLyricIndex() {
  if (lyrics.value.length === 0) return;
  const t = audio.currentTime;
  for (let i = lyrics.value.length - 1; i >= 0; i--) {
    if (lyrics.value[i].time <= t) {
      currentLyricIdx.value = i;
      return;
    }
  }
  currentLyricIdx.value = -1;
}

// 波形生成（优先 ffmpeg 快速路径 → 自动缓存 → 回退 symphonia）
async function loadWaveform(track: Track) {
  waveformPeaks.value = [];
  if (!track.path || !isTauri()) return;

  const currentAbortId = ++waveformAbortId;
  waveformLoading.value = true;

  try {
    const result = await invoke<{ peaks: number[] }>('generate_waveform_fast', {
      path: track.path,
      numPeaks: 1500,
    });
    if (currentAbortId === waveformAbortId) {
      waveformPeaks.value = result.peaks;
    }
  } catch (e) {
    console.warn('[waveform] generation failed:', e);
    if (currentAbortId === waveformAbortId) {
      waveformPeaks.value = [];
    }
  } finally {
    if (currentAbortId === waveformAbortId) {
      waveformLoading.value = false;
    }
  }
}

function onWaveformSeek(time: number) {
  audio.currentTime = time;
}

// 音频详情
async function loadAudioInfo(track: Track) {
  audioInfo.value = null;
  const currentAbortId = ++infoAbortId;
  if (!track.path || !isTauri()) return;

  try {
    const result = await invoke<AudioInfo>('get_audio_info', { path: track.path });
    if (currentAbortId !== infoAbortId) return;
    audioInfo.value = result;
  } catch (e) {
    if (currentAbortId !== infoAbortId) return;
    console.warn('[audio-info] failed:', e);
  }

  // Analyze loudness in background
  try {
    const loud = await invoke<{ offsetNum: number; integrated: string }>('analyze_loudness', { path: track.path });
    if (currentAbortId !== infoAbortId) return;
    if (audioInfo.value && loud.offsetNum !== 0) {
      const gainDb = Math.max(-10, Math.min(10, loud.offsetNum));
      const gainLinear = Math.pow(10, gainDb / 20);
      (audioInfo.value as any)._loudnessGain = gainLinear;
      // 立即生效
      setMasterVolume(volume.value * gainLinear);
    }
  } catch (e) {
    if (currentAbortId !== infoAbortId) return;
    console.warn('[loudness] analysis failed:', e);
  }
}

// 提取专辑封面
async function loadCoverArt(track: Track) {
  coverArtUrl.value = null;
  coverLoaded.value = false;
  if (!track.path || !isTauri()) return;

  try {
    const result = await invoke<string | null>('extract_cover_art', { path: track.path });
    if (result) {
      const url = convertFileSrc(result);
      // Verify the image actually loads
      const img = new Image();
      img.onload = () => {
        coverArtUrl.value = url;
        coverLoaded.value = true;
      };
      img.onerror = () => {
        coverArtUrl.value = null;
        coverLoaded.value = true;
      };
      img.src = url;
    } else {
      coverLoaded.value = true;
    }
  } catch (e) {
    console.warn('[cover] extraction failed:', e);
    coverLoaded.value = true;
  }
}

function toggleSleepTimer() {
  if (sleepTimer) {
    clearTimeout(sleepTimer);
    sleepTimer = null;
    sleepMinutes.value = 0;
    showToast('定时器已取消', 'info');
    return;
  }
  const mins = sleepMinutes.value === 0 ? 15 : sleepMinutes.value === 15 ? 30 : sleepMinutes.value === 30 ? 60 : 0;
  sleepMinutes.value = mins;
  if (mins === 0) return;
  sleepTimer = setTimeout(() => {
    pause();
    sleepTimer = null;
    sleepMinutes.value = 0;
    showToast('⏰ 睡眠定时结束，已暂停播放', 'info');
  }, mins * 60 * 1000);
  showToast(`将在 ${mins} 分钟后自动暂停`, 'info');
}

// 播放列表筛选: 'all' | 'favorites' | 'history'
const playlistFilter = ref<'all' | 'favorites' | 'history'>('all');
const filteredPlaylist = computed(() => {
  if (playlistFilter.value === 'favorites') {
    return favorites.value.map(track => ({
      track,
      idx: playlist.value.findIndex(p => p.path === track.path || p.url === track.url),
    }));
  }
  if (playlistFilter.value === 'history') {
    return playHistory.value.map(h => {
      const idx = playlist.value.findIndex(p => p.name === h.name);
      const track = idx >= 0 ? playlist.value[idx] : { name: h.name, url: '', path: '' };
      return { track, idx, time: h.time };
    });
  }
  return playlist.value.map((track, idx) => ({ track, idx }));
});

const currentTrack = computed(() => playlist.value[currentIndex.value] ?? null);
const coverLetter = computed(() => {
  return currentTrack.value?.name?.trim()?.charAt(0)?.toUpperCase() || 'M';
});

// 封面图片 URL（专辑封面优先，否则显示首字母）
const coverArtUrl = ref<string | null>(null);
const coverLoaded = ref(false);

async function addFiles(files: FileList | File[]) {
  const arr = Array.isArray(files) ? files : Array.from(files);
  for (const f of arr) {
    const filePath = isTauri() ? ((f as any).path as string | undefined) : undefined;
    if (filePath) {
      // Has native path — use the fast path
      await addPaths([filePath]);
      continue;
    }
    // No native path available — blob URL only, can't persist
    const blobUrl = URL.createObjectURL(f);
    playlist.value.push({ name: f.name, url: blobUrl, persistent: false });
  }
  if (currentIndex.value === -1 && playlist.value.length > 0) {
    currentIndex.value = 0;
    loadCurrent();
  }
  await saveState();
}

function handleFiles(e: Event) {
  const input = e.target as HTMLInputElement;
  if (!input.files) return;
  void addFiles(input.files);
  input.value = '';
}

function openAudioPicker() {
  // Use Tauri's native file dialog — gives real filesystem paths
  open({
    multiple: true,
    filters: [{ name: 'Audio', extensions: ['mp3', 'flac', 'wav', 'ogg', 'aac', 'm4a', 'wma'] }],
  }).then((selected) => {
    if (selected && selected.length > 0) {
      addPaths(selected as string[]);
    }
  });
}

async function addPaths(paths: string[]) {
  for (const filePath of paths) {
    const fname = filePath.split('/').pop() || filePath.split('\\').pop() || filePath;
    try {
      // Copy to app data dir for persistence, then use convertFileSrc for URL
      let stablePath: string;
      try {
        stablePath = await invoke<string>('copy_file_to_data', { source: filePath });
      } catch {
        stablePath = filePath;
      }
      const url = convertFileSrc(stablePath);
      playlist.value.push({ name: fname, url, path: stablePath, persistent: true });
      console.log('[add] imported:', fname, '→', stablePath);
    } catch (e) {
      showToast(`导入失败: ${fname}`, 'error');
    }
  }
  if (currentIndex.value === -1 && playlist.value.length > 0) {
    currentIndex.value = 0;
    loadCurrent();
  }
  await saveState();
}

function loadCurrent() {
  const item = playlist.value[currentIndex.value];
  if (!item) return;
  // 先彻底停止旧音源，防止多首歌同时播放
  audio.pause();
  audio.removeAttribute('src');
  audio.load();
  // 再加载新音源
  audio.src = item.url;
  audio.playbackRate = playbackRate.value;
  audio.load();
  loadLyrics(item);
  loadWaveform(item);
  loadAudioInfo(item);
  loadCoverArt(item);
  setupMediaSession(item);
}

// Media Session API — 每首歌生成独有专辑封面
function generateTrackArtwork(trackName: string): Promise<Blob> {
  const size = 256;
  const canvas = document.createElement('canvas');
  canvas.width = size; canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  let hash = 0;
  for (let i = 0; i < trackName.length; i++) hash = ((hash << 5) - hash) + trackName.charCodeAt(i);
  const hue = Math.abs(hash % 360);
  const grad = ctx.createLinearGradient(0, 0, size, size);
  grad.addColorStop(0, `hsl(${hue}, 60%, 25%)`);
  grad.addColorStop(1, `hsl(${(hue + 40) % 360}, 50%, 18%)`);
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, size, size);

  const letter = trackName.trim().charAt(0).toUpperCase() || '♪';
  ctx.fillStyle = 'rgba(255,255,255,0.9)';
  ctx.font = 'bold 120px "Inter", "SF Pro Display", sans-serif';
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  ctx.fillText(letter, size / 2, size / 2);

  const displayName = trackName.replace(/\.[^.]+$/, '').substring(0, 20);
  ctx.fillStyle = 'rgba(255,255,255,0.45)';
  ctx.font = '16px "Inter", "SF Pro Display", sans-serif';
  ctx.fillText(displayName, size / 2, size - 30);

  return new Promise(resolve => canvas.toBlob(b => resolve(b!), 'image/png'));
}

let mediaSessionReady = false;

async function setupMediaSession(track: Track) {
  if (!('mediaSession' in navigator)) return;
  
  const name = track.name.replace(/\.[^.]+$/, '');
  const dashIdx = name.indexOf(' - ');
  const artist = dashIdx > 0 ? name.substring(0, dashIdx) : 'Hanono';
  const title = dashIdx > 0 ? name.substring(dashIdx + 3) : name;

  if (!mediaSessionReady) {
    navigator.mediaSession.setActionHandler('play', () => play());
    navigator.mediaSession.setActionHandler('pause', () => pause());
    navigator.mediaSession.setActionHandler('previoustrack', () => prev());
    navigator.mediaSession.setActionHandler('nexttrack', () => next());
    mediaSessionReady = true;
  }

  try {
    const blob = await generateTrackArtwork(track.name);
    const blobUrl = URL.createObjectURL(blob);
    navigator.mediaSession.metadata = new MediaMetadata({
      title, artist, album: 'Hanono',
      artwork: [{ src: blobUrl, sizes: '256x256', type: 'image/png' }],
    });
  } catch {
    navigator.mediaSession.metadata = new MediaMetadata({ title, artist, album: 'Hanono' });
  }
}

async function play() {
  if (playlist.value.length === 0) return;
  ensureAudioContext();
  if (!audio.src && currentIndex.value === -1) {
    currentIndex.value = 0;
    loadCurrent();
  }
  if (!audio.src) return;
  try {
    await audio.play();
  } catch (err) {
    isPlaying.value = false;
    console.warn('Play failed:', err);
  }
}

function pause() {
  audio.pause();
}

function togglePlay() {
  if (isPlaying.value) { pause(); }
  else if (playlist.value.length > 0) { play(); }
}

function next() {
  if (playlist.value.length === 0) return;
  if (shuffle.value && playlist.value.length > 1) {
    let idx = currentIndex.value;
    while (idx === currentIndex.value) {
      idx = Math.floor(Math.random() * playlist.value.length);
    }
    currentIndex.value = idx;
    loadCurrent();
    play();
    return;
  }

  if (repeatMode.value === 'one') {
    loadCurrent();
    play();
    return;
  }

  if (currentIndex.value + 1 < playlist.value.length) {
    currentIndex.value += 1;
    loadCurrent();
    play();
    return;
  }

  if (repeatMode.value === 'all') {
    currentIndex.value = 0;
    loadCurrent();
    play();
  }
}

function prev() {
  if (playlist.value.length === 0) return;
  if (shuffle.value && playlist.value.length > 1) {
    let idx = currentIndex.value;
    while (idx === currentIndex.value) {
      idx = Math.floor(Math.random() * playlist.value.length);
    }
    currentIndex.value = idx;
    loadCurrent();
    play();
    return;
  }

  if (currentIndex.value - 1 >= 0) currentIndex.value -= 1;
  else currentIndex.value = playlist.value.length - 1;
  loadCurrent();
  play();
}

function seekTo(v: number) {
  audio.currentTime = v;
}

function onSeekEvent(e: Event) {
  const v = Number((e.target as HTMLInputElement).value || 0);
  if (isSeeking.value) {
    seekPreview.value = v;
  } else {
    seekTo(v);
  }
}

function onSeekPointerDown(e: PointerEvent) {
  isSeeking.value = true;
  seekPreview.value = Number((e.target as HTMLInputElement).value || 0);
}

function onSeekPointerUp(e: PointerEvent) {
  const v = Number((e.target as HTMLInputElement).value || 0);
  isSeeking.value = false;
  seekTo(v);
}

function selectTrack(i: number) {
  if (i < 0 || i >= playlist.value.length) return;
  if (i === currentIndex.value) {
    if (!isPlaying.value) play();
    return;
  }
  currentIndex.value = i;
  recordPlay(playlist.value[i]);
  loadCurrent();
  play();
}

// 从收藏列表播放：先在主列表找，找不到则临时加入
function selectFavoritesTrack(favIndex: number) {
  const fav = favorites.value[favIndex];
  if (!fav) return;
  let idx = playlist.value.findIndex(p => p.path === fav.path || p.url === fav.url);
  if (idx < 0) {
    // 该曲目不在主列表中，临时加入
    idx = playlist.value.length;
    playlist.value.push({ ...fav });
  }
  selectTrack(idx);
  recordPlay(playlist.value[idx]);
}

// 从收藏列表移除（仅移除收藏，不影响主列表）
function removeFromFavorites(favIndex: number) {
  if (favIndex < 0 || favIndex >= favorites.value.length) return;
  // 如果当前正在播放该曲目，且主列表中已不存在，则停止
  const fav = favorites.value[favIndex];
  const inPlaylist = playlist.value.findIndex(p => p.path === fav.path || p.url === fav.url);
  if (inPlaylist === currentIndex.value && inPlaylist < 0) {
    audio.pause();
    audio.removeAttribute('src');
    audio.load();
    isPlaying.value = false;
    currentTime.value = 0;
    duration.value = 0;
  }
  favorites.value.splice(favIndex, 1);
}

function removeTrack(i: number) {
  const wasCurrent = i === currentIndex.value;
  playlist.value.splice(i, 1);
  if (playlist.value.length === 0) {
    currentIndex.value = -1;
    audio.pause();
    audio.removeAttribute('src');
    audio.load();
    isPlaying.value = false;
    currentTime.value = 0;
    duration.value = 0;
    return;
  }
  if (wasCurrent) {
    if (i >= playlist.value.length) currentIndex.value = playlist.value.length - 1;
    loadCurrent();
    if (isPlaying.value) play();
  } else if (i < currentIndex.value) {
    currentIndex.value -= 1;
  }
}

function clearPlaylist() {
  playlist.value.length = 0; 
  currentIndex.value = -1;
  audio.pause();
  audio.removeAttribute('src');
  audio.load();
  isPlaying.value = false;
  currentTime.value = 0;
  duration.value = 0;
}

function toggleShuffle() {
  shuffle.value = !shuffle.value;
}

function cycleRepeat() {
  if (repeatMode.value === 'off') repeatMode.value = 'all';
  else if (repeatMode.value === 'all') repeatMode.value = 'one';
  else repeatMode.value = 'off';
}

// 收藏/取消收藏
function isFavorited(track: Track): boolean {
  return favorites.value.some(f => f.path === track.path || f.url === track.url);
}

function toggleFavor(index: number) {
  const track = playlist.value[index];
  if (!track) return;
  const idx = favorites.value.findIndex(f => f.path === track.path || f.url === track.url);
  if (idx >= 0) {
    favorites.value.splice(idx, 1);
  } else {
    favorites.value.push({ ...track });
  }
}

// 右键菜单
function onTrackContextMenu(e: MouseEvent, index: number) {
  e.preventDefault();
  contextMenu.value = { visible: true, x: e.clientX, y: e.clientY, trackIndex: index };
}

function closeContextMenu() {
  contextMenu.value.visible = false;
}

function onContextMenuAction(action: string) {
  const idx = contextMenu.value.trackIndex;
  if (idx < 0 || idx >= playlist.value.length) { closeContextMenu(); return; }
  switch (action) {
    case 'play':
      selectTrack(idx);
      break;
    case 'favor':
      toggleFavor(idx);
      break;
    case 'convert':
      convertingTrackName.value = playlist.value[idx].name;
      convertingTrackPath.value = playlist.value[idx].path || '';
      showConvertDialog.value = true;
      closeContextMenu();
      return;
    case 'trim':
      trimTrackName.value = playlist.value[idx].name;
      trimTrackPath.value = playlist.value[idx].path || '';
      showTrimDialog.value = true;
      closeContextMenu();
      return;
    case 'remove':
      removeTrack(idx);
      break;
    case 'reveal':
      revealInFinder(playlist.value[idx].path);
      return;
  }
  closeContextMenu();
}

// 格式转换
async function doConvert(format: string, bitrate: string) {
  showConvertDialog.value = false;
  if (!convertingTrackPath.value || !isTauri()) {
    showToast('无法转换（仅支持本地文件）', 'error');
    return;
  }
  showToast('正在转换...', 'info');
  try {
    const outPath = await invoke<string>('convert_audio', {
      source: convertingTrackPath.value,
      format,
      bitrate,
    });
    showToast(`转换完成: ${outPath.split('/').pop()}`, 'success');
  } catch (e) {
    console.error('[convert] failed:', e);
    showToast('转换失败', 'error');
  }
}

// 音频裁剪
async function doTrim(startSec: number, endSec: number) {
  showTrimDialog.value = false;
  if (!trimTrackPath.value || !isTauri()) {
    showToast('无法裁剪（仅支持本地文件）', 'error');
    return;
  }
  showToast('正在裁剪...', 'info');
  try {
    const outPath = await invoke<string>('trim_audio', {
      source: trimTrackPath.value,
      startSec,
      endSec,
    });
    showToast(`裁剪完成: ${outPath.split('/').pop()}`, 'success');
  } catch (e) {
    console.error('[trim] failed:', e);
    showToast('裁剪失败', 'error');
  }
}

// 在文件管理器中显示
async function revealInFinder(path: string | undefined) {
  closeContextMenu();
  if (!path || !isTauri()) return;
  try {
    await invoke('reveal_in_finder', { path });
  } catch (e) {
    console.warn('[reveal] failed:', e);
  }
}

const STORAGE_KEY = 'hanono_playlist_v2';

async function saveState() {
  try {
    const data = {
      playlist: playlist.value.map(p => ({ name: p.name, url: p.url, path: p.path })),
      favorites: favorites.value.map(f => ({ name: f.name, url: f.url, path: f.path })),
      history: playHistory.value,
      currentIndex: currentIndex.value,
      shuffle: shuffle.value,
      repeatMode: repeatMode.value,
    };
    const json = JSON.stringify(data);
    localStorage.setItem(STORAGE_KEY, json);
    if (isTauri()) {
      try {
        await invoke('save_playlist', { data: JSON.stringify({ playlist: data.playlist, currentIndex: data.currentIndex, shuffle: data.shuffle, repeatMode: data.repeatMode }) });
        await invoke('save_favorites', { data: JSON.stringify(data.favorites) });
      } catch (e) {
        console.error('[state] save failed:', e);
        showToast('保存数据失败', 'error');
      }
    }
  } catch (e) {
    console.error('[state] saveState error:', e);
  }
}

async function loadState() {
  isRestoring.value = true;
  try {
    let raw: string | null = null;
    let favRaw: string | null = null;

    if (isTauri()) {
      try { raw = await invoke<string>('load_playlist'); } catch { /* ignore */ }
      try { favRaw = await invoke<string>('load_favorites'); } catch { /* ignore */ }
    }
    if (!raw) raw = localStorage.getItem(STORAGE_KEY);

    if (raw) {
      const data = JSON.parse(raw);
      await applyState(data);
      if (data.history && Array.isArray(data.history)) {
        playHistory.value = data.history;
      }
    }
    if (favRaw) {
      const favData = JSON.parse(favRaw);
      if (Array.isArray(favData)) {
        for (const f of favData) {
          const url = await resolvePlaylistUrl(f);
          if (url) {
            favorites.value.push({ name: f.name || f.path, url, path: f.path, persistent: Boolean(f.path) });
          }
        }
      }
    }
  } catch (e) {
    console.error('[state] loadState error:', e);
  } finally {
    isRestoring.value = false;
  }
}

async function resolvePlaylistUrl(entry: any) {
  if (isTauri() && entry.path && typeof entry.path === 'string') {
    try {
      const url = convertFileSrc(entry.path);
      console.log('[resolve] OK:', entry.name);
      return url;
    } catch (e) {
      console.warn('[resolve] convertFileSrc failed for:', entry.path, e);
    }
    return '';
  }
  // Browser fallback: blob URLs are ephemeral
  const url = entry.url || '';
  if (url.startsWith('blob:')) return '';
  return url;
}

async function applyState(data: any) {
  if (!data) return;
  playlist.value.length = 0;

  if (data.playlist && Array.isArray(data.playlist)) {
    for (const p of data.playlist) {
      const url = await resolvePlaylistUrl(p);
      if (url) {
        playlist.value.push({
          name: p.name || p.path || p.url,
          url,
          path: p.path,
          persistent: Boolean(p.path),
        });
      }
    }
  }
  if (typeof data.currentIndex === 'number') {
    currentIndex.value = Math.min(Math.max(data.currentIndex, -1), playlist.value.length - 1);
  }
  if (typeof data.shuffle === 'boolean') shuffle.value = data.shuffle;
  if (data.repeatMode) repeatMode.value = data.repeatMode;
  if (currentIndex.value === -1 && playlist.value.length > 0) {
    currentIndex.value = 0;
  }
  if (currentIndex.value >= 0) loadCurrent();
}

onMounted(async () => {
  audio.volume = volume.value;
  audio.addEventListener('timeupdate', () => {
    currentTime.value = audio.currentTime;
    duration.value = audio.duration || 0;
    updateLyricIndex();
  });
  audio.addEventListener('play', () => (isPlaying.value = true));
  audio.addEventListener('pause', () => (isPlaying.value = false));
  audio.addEventListener('ended', () => {
    if (currentIndex.value + 1 < playlist.value.length) next();
    else isPlaying.value = false;
  });

  const onKey = (ev: KeyboardEvent) => {
    if (ev.code === 'Space') {
      ev.preventDefault();
      togglePlay();
    } else if (ev.key === 'ArrowRight') {
      next();
    } else if (ev.key === 'ArrowLeft') {
      prev();
    } else if (ev.code === 'KeyS') {
      toggleShuffle();
    } else if (ev.code === 'KeyR') {
      cycleRepeat();
    } else if (ev.code === 'KeyO') {
      openAudioPicker();
    } else if (ev.code === 'KeyF') {
      // 收藏当前曲目
      if (currentIndex.value >= 0) toggleFavor(currentIndex.value);
    } else if (ev.code === 'Slash' || ev.code === 'KeyH') {
      // 显示/隐藏快捷键面板
      showShortcuts.value = !showShortcuts.value;
    }
  };
  window.addEventListener('keydown', onKey);

  // 点击其他区域关闭右键菜单
  const onClickOutside = () => { if (contextMenu.value.visible) closeContextMenu(); };
  window.addEventListener('click', onClickOutside);

  // Register cleanup BEFORE any await (Vue requirement for async setup)
  let unlisten: (() => void) | undefined;
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey);
    window.removeEventListener('click', onClickOutside);
    if (unlisten) unlisten();
    if (sleepTimer) clearTimeout(sleepTimer);
  });

  // Tauri native drag-drop
  if (isTauri()) {
    const AUDIO_EXTS = new Set(['mp3','flac','wav','ogg','aac','m4a','wma','opus','aiff','wv']);
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'drop' && event.payload.paths.length > 0) {
        const audioPaths = event.payload.paths.filter(p => {
          const ext = p.split('.').pop()?.toLowerCase() || '';
          return AUDIO_EXTS.has(ext);
        });
        if (audioPaths.length > 0) {
          console.log('[drop]', audioPaths.length, 'audio files');
          addPaths(audioPaths);
        }
      }
    });

    // Listen for system menu actions
    listen<string>('menu-action', (event) => {
      switch (event.payload) {
        case 'play_pause': togglePlay(); break;
        case 'next': next(); break;
        case 'prev': prev(); break;
        case 'shuffle': toggleShuffle(); break;
        case 'repeat': cycleRepeat(); break;
        case 'vol_up': setVolume(Math.min(1, volume.value + 0.1)); break;
        case 'vol_down': setVolume(Math.max(0, volume.value - 0.1)); break;
        case 'import': openAudioPicker(); break;
        case 'favorite':
          if (currentIndex.value >= 0) toggleFavor(currentIndex.value);
          break;
        case 'clear': clearPlaylist(); break;
        case 'about': showAbout.value = true; break;
      }
    });
  }

  loadState();
});

watch([
  playlist,
  favorites,
  shuffle,
  repeatMode,
  currentIndex,
], () => {
  if (!isRestoring.value) saveState();
}, { deep: true });

function formatTime(s: number) {
  if (!s || Number.isNaN(s)) return '0:00';
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60).toString().padStart(2, '0');
  return `${m}:${sec}`;
}
</script>

<template>
  <main class="player-shell" :class="{ 'is-playing': isPlaying, 'is-mini': isMini }">
    <div class="shell-inner">
    <!-- macOS 原生标题栏占位：红绿灯区域 + 拖拽区域 -->
    <div class="titlebar" data-tauri-drag-region></div>
    <!-- 顶部导航栏 -->
    <section class="header-panel">
      <div class="header-brand">
        <div class="brand-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
            <rect x="5" y="6" width="2.2" height="12" rx="1.1" fill="currentColor" opacity="0.6"/>
            <rect x="8.6" y="2" width="2.6" height="20" rx="1.3" fill="currentColor"/>
            <rect x="12.5" y="4.5" width="3" height="4" rx="1.5" fill="currentColor"/>
            <rect x="16.8" y="2" width="2.6" height="20" rx="1.3" fill="currentColor"/>
            <rect x="20.8" y="6" width="2.2" height="12" rx="1.1" fill="currentColor" opacity="0.6"/>
          </svg>
        </div>
        <div class="header-title">
          <strong>Hanono</strong>
          <span>沉浸式音乐体验</span>
        </div>
      </div>
      <div class="header-actions">
        <button type="button" class="action-btn ghost" @click="toggleMini" :title="isMini ? '完整模式' : 'Mini 模式'">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M8 3H5a2 2 0 00-2 2v3m18 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M3 16v3a2 2 0 002 2h3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
          {{ isMini ? '完整' : 'Mini' }}
        </button>
        <button type="button" class="action-btn ghost" @click="showAbout = true" title="关于 Hanono">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" stroke="currentColor" stroke-width="2"/><path d="M12 16v-4M12 8h.01" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
          关于
        </button>
        <button type="button" class="action-btn primary" @click="openAudioPicker">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M12 5v14m-7-7h14" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
          导入音频
        </button>
        <button type="button" class="action-btn danger" @click="clearPlaylist">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2m3 0v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6h14z" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>
          清空
        </button>
      </div>
    </section>

    <!-- 主内容区 -->
    <section class="player-grid">
      <!-- 播放器卡片 -->
      <article class="player-card">
        <!-- 封面区域 -->
        <div class="cover-section">
          <div class="cover-art-wrapper"
            :class="{
              empty: !currentTrack,
              playing: isPlaying,
              ['effect-' + coverEffect]: true
            }"
            @click="cycleCoverEffect"
            :title="'封面动效: ' + effectLabels[coverEffect] + ' (点击切换)'"
          >
            <div class="cover-art" :class="{ 'has-cover': coverArtUrl }">
              <img v-if="coverArtUrl" :src="coverArtUrl" class="cover-img" alt="专辑封面" />
              <span v-else-if="currentTrack" class="cover-letter">{{ coverLetter }}</span>
              <svg v-else class="cover-placeholder" width="64" height="64" viewBox="0 0 24 24" fill="none">
                <path d="M9 18V5l12-2v13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                <circle cx="6" cy="18" r="3" stroke="currentColor" stroke-width="1.5"/>
                <circle cx="18" cy="16" r="3" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </div>
            <div v-if="isPlaying && currentTrack" class="cover-glow"></div>
          </div>
          <div class="track-info">
            <div class="track-name-row">
              <p class="track-name" :class="{ marquee: isPlaying }">{{ currentTrack?.name || '等待音乐响起' }}</p>
              <button
                v-if="currentTrack"
                class="favor-btn"
                :class="{ favored: isFavorited(currentTrack) }"
                @click="toggleFavor(currentIndex)"
                :title="isFavorited(currentTrack) ? '取消收藏' : '收藏'"
              >
                <svg width="18" height="18" viewBox="0 0 24 24" :fill="isFavorited(currentTrack) ? 'currentColor' : 'none'">
                  <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
            </div>
            <p class="track-meta">
              <template v-if="currentTrack">
                <span class="meta-badge">{{ currentIndex + 1 }} / {{ playlist.length }}</span>
                <span class="meta-dot">·</span>
                <span>{{ formatTime(duration) }}</span>
                <template v-if="currentFxLabel">
                  <span class="meta-dot">·</span>
                  <span class="fx-mode-tag inline-fx-tag">{{ currentFxLabel }}</span>
                </template>
              </template>
              <template v-else>
                拖拽音频文件或点击导入开始
              </template>
            </p>
          </div>
        </div>

        <!-- 歌词显示 -->
        <div v-if="lyrics.length > 0 && !isMini" class="lyrics-section">
          <p class="lyric-line active" v-if="currentLyricIdx >= 0">{{ lyrics[currentLyricIdx]?.text }}</p>
          <p class="lyric-line next" v-if="currentLyricIdx + 1 < lyrics.length">{{ lyrics[currentLyricIdx + 1]?.text }}</p>
        </div>

        <!-- 波形图 -->
        <div v-if="currentTrack && !isMini" class="waveform-section">
          <WaveformCanvas
            :peaks="waveformPeaks"
            :currentTime="currentTime"
            :duration="duration"
            :loading="waveformLoading"
            @seek="onWaveformSeek"
          />
        </div>

        <!-- 音频信息标签行 -->
        <div v-if="audioInfo && !isMini" class="audio-tags">
          <span class="audio-tag codec">{{ audioInfo.codec }}</span>
          <span class="audio-tag">{{ audioInfo.sampleRateStr }}</span>
          <span class="audio-tag">{{ audioInfo.bitrateStr }}</span>
          <span class="audio-tag">{{ audioInfo.channelsStr }}</span>
          <span v-if="audioInfo.bitDepth > 0" class="audio-tag">{{ audioInfo.bitDepthStr }}</span>
          <span class="audio-tag dim">{{ audioInfo.fileSizeStr }}</span>
        </div>

        <!-- 播放控制区 -->
        <div class="player-controls">
          <div class="progress-section">
            <div class="progress-bar-wrap">
              <div class="progress-bar-track">
                <div class="progress-bar-fill" :style="{ width: (duration ? ((isSeeking ? seekPreview : currentTime) / duration) * 100 : 0) + '%' }"></div>
                <input class="seek-input" type="range" :max="duration || 0" step="0.1" :value="isSeeking ? seekPreview : currentTime" @pointerdown="onSeekPointerDown" @pointerup="onSeekPointerUp" @input="onSeekEvent" />
              </div>
              <div class="time-display">
                <span class="time-current">{{ formatTime(isSeeking ? seekPreview : currentTime) }}</span>
                <span class="time-divider">/</span>
                <span class="time-total">{{ formatTime(duration) }}</span>
              </div>
            </div>
          </div>

          <div class="control-bar">
            <div class="playback-group">
              <button type="button" class="ctrl-btn icon-btn" :class="{ active: shuffle }" @click="toggleShuffle" title="随机播放 (S)">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
              </button>
              <button type="button" class="ctrl-btn icon-btn" @click="prev" title="上一首 (←)">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none"><path d="M19 20L9 12l10-8v16z" fill="currentColor"/><path d="M5 19V5" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
              </button>
              <button type="button" class="ctrl-btn play-btn-lg" @click="togglePlay" title="播放/暂停 (空格)">
                <svg v-if="!isPlaying" width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M5 3l14 9-14 9V3z" fill="currentColor"/></svg>
                <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="none"><rect x="5" y="3" width="5" height="18" rx="1.5" fill="currentColor"/><rect x="14" y="3" width="5" height="18" rx="1.5" fill="currentColor"/></svg>
              </button>
              <button type="button" class="ctrl-btn icon-btn" @click="next" title="下一首 (→)">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none"><path d="M5 4l10 8-10 8V4z" fill="currentColor"/><path d="M19 5v14" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
              </button>
              <button type="button" class="ctrl-btn icon-btn" :class="{ active: repeatMode !== 'off' }" @click="cycleRepeat" title="循环模式 (R)">
                <svg v-if="repeatMode === 'off'" width="18" height="18" viewBox="0 0 24 24" fill="none" opacity="0.5"><path d="M17 1l4 4-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M3 11V9a4 4 0 014-4h14M7 23l-4-4 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M21 13v2a4 4 0 01-4 4H3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                <svg v-else-if="repeatMode === 'all'" width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M17 1l4 4-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M3 11V9a4 4 0 014-4h14M7 23l-4-4 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M21 13v2a4 4 0 01-4 4H3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
                <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none"><path d="M17 1l4 4-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M3 11V9a4 4 0 014-4h14M7 23l-4-4 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M21 13v2a4 4 0 01-4 4H3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><text x="12" y="16" text-anchor="middle" font-size="9" font-weight="700" fill="currentColor">1</text></svg>
              </button>
            </div>
            <div class="util-group">
              <div class="volume-inline">
                <button type="button" class="volume-icon-btn" @click="toggleMute" :title="volume === 0 ? '取消静音' : '静音'">
                  <svg v-if="volume === 0" width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M11 5L6 9H2v6h4l5 4V5z" fill="currentColor"/><path d="M23 9l-6 6M17 9l6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>
                  <svg v-else-if="volume < 0.5" width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M11 5L6 9H2v6h4l5 4V5z" fill="currentColor"/><path d="M17.07 6.93a6 6 0 010 10.14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
                  <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M11 5L6 9H2v6h4l5 4V5z" fill="currentColor"/><path d="M19.07 4.93a10 10 0 010 14.14M17.07 6.93a6 6 0 010 10.14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
                </button>
                <div class="volume-slider-track">
                  <div class="volume-slider-fill" :style="{ width: (volume * 100) + '%' }"></div>
                  <input class="volume-slider-input" type="range" min="0" max="1" step="0.01" :value="volume" @input="onVolumeInput" />
                </div>
                <span class="volume-pct">{{ Math.round(volume * 100) }}</span>
              </div>
              <button type="button" class="shortcut-toggle" :class="{ active: playbackRate !== 1 }" @click="cycleSpeed" :title="`播放速度 ${playbackRate}x`">
                <span style="font-size:11px;font-weight:700">{{ playbackRate }}x</span>
              </button>
              <button type="button" class="shortcut-toggle" :class="{ active: sleepMinutes > 0 }" @click="toggleSleepTimer" :title="sleepMinutes > 0 ? `定时 ${sleepMinutes} 分钟 (点击取消)` : '睡眠定时器'">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2"/><path d="M12 6v6l4 2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>
              </button>
              <button type="button" class="shortcut-toggle fx-btn" :class="{ active: currentPreset !== 'flat' || bassBoost > 0 || surroundAmount > 0 || reverbAmount > 0 }" @click="openFxPanel()" title="音效增强">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M3 17h2l3-13h8l3 13h2" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><circle cx="8" cy="8" r="1.5" fill="currentColor"/><circle cx="16" cy="6" r="1.5" fill="currentColor"/><circle cx="12" cy="11" r="1.5" fill="currentColor"/></svg>
              </button>
              <button type="button" class="shortcut-toggle" @click="showShortcuts = !showShortcuts" title="快捷键 (?)">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" stroke="currentColor" stroke-width="2"/><path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><circle cx="12" cy="17" r="1" fill="currentColor"/></svg>
              </button>
              <span v-if="favorites.length > 0" class="favor-count">
                ❤️ {{ favorites.length }}
              </span>
            </div>
          </div>
        </div>
      </article>

      <!-- 播放列表 -->
      <PlaylistSidebar
        :playlist="playlist"
        :favorites="favorites"
        :playHistory="playHistory"
        :currentIndex="currentIndex"
        :isPlaying="isPlaying"
        :playlistFilter="playlistFilter"
        :filteredPlaylist="filteredPlaylist"
        @update:playlistFilter="playlistFilter = $event"
        @selectTrack="selectTrack"
        @selectFavoritesTrack="selectFavoritesTrack"
        @removeTrack="removeTrack"
        @removeFromFavorites="removeFromFavorites"
        @removeHistory="(i: number) => playHistory.splice(i, 1)"
        @clearHistory="playHistory.length = 0; showToast('播放历史已清空', 'success')"
        @contextMenu="onTrackContextMenu"
      />
    </section>

    </div>
    <input ref="fileInputRef" type="file" accept="audio/*" multiple @change="handleFiles" class="hidden-input" />

    <!-- Toast 通知 -->
    <Teleport to="body">
      <div class="toast-container">
        <transition-group name="toast">
          <div v-for="t in toasts" :key="t.id" class="toast-item" :class="'toast-' + t.type">
            <span>{{ t.message }}</span>
          </div>
        </transition-group>
      </div>
    </Teleport>

    <!-- 右键菜单 + 快捷键 + 关于 -->
    <AppModals
      :showShortcuts="showShortcuts"
      :showAbout="showAbout"
      :contextMenu="contextMenu"
      @closeShortcuts="showShortcuts = false"
      @closeAbout="showAbout = false"
      @contextMenuAction="onContextMenuAction"
      @closeContextMenu="closeContextMenu"
    />

    <!-- 音效面板 -->
    <FxPanel
      :show="showFxPanel"
      :eqBands="eqBands"
      :bassBoost="bassBoost"
      :surroundAmount="surroundAmount"
      :reverbAmount="reverbAmount"
      :currentPreset="currentPreset"
      :eqPresets="eqPresets"
      @close="showFxPanel = false"
      @applyPreset="applyPreset"
      @onBassBoostInput="onBassBoostInput"
      @onSurroundInput="onSurroundInput"
      @onReverbInput="onReverbInput"
      @onEqBandInput="onEqBandInput"
      @resetAll="resetAllEffects"
    />
    <!-- 格式转换对话框 -->
    <ConvertDialog
      :show="showConvertDialog"
      :trackName="convertingTrackName"
      @close="showConvertDialog = false"
      @convert="doConvert"
    />
    <!-- 音频裁剪对话框 -->
    <TrimDialog
      :show="showTrimDialog"
      :trackName="trimTrackName"
      :duration="duration"
      @close="showTrimDialog = false"
      @trim="doTrim"
    />
  </main>
</template>

<style scoped>
/* ========== CSS 自定义属性 ========== */
.player-shell {
  --accent: #6366f1;
  --accent-2: #8b5cf6;
  --accent-3: #a78bfa;
  --bg-base: #0f0f1a;
  --bg-surface: rgba(255, 255, 255, 0.06);
  --bg-card: rgba(255, 255, 255, 0.07);
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
  --border-subtle: rgba(255, 255, 255, 0.08);
  --border-medium: rgba(255, 255, 255, 0.12);
  --radius-sm: 10px;
  --radius-md: 16px;
  --radius-lg: 24px;
  --radius-xl: 28px;
  user-select: none;
  -webkit-user-select: none;
}

/* ========== 全局布局 ========== */
.player-shell {
  height: 100vh;
  width: 100%;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  background: var(--bg-base);
  color: var(--text-primary);
  font-family: 'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  position: relative;
  overflow: hidden;
}

.shell-inner {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 6px 10px;
  min-height: 0;
  overflow: hidden;
}

.player-shell::before {
  content: '';
  position: absolute;
  top: -30%;
  left: -20%;
  width: 80%;
  height: 80%;
  background: radial-gradient(circle, rgba(99, 102, 241, 0.12) 0%, transparent 70%);
  pointer-events: none;
  transition: all 0.8s ease;
}

.player-shell::after {
  content: '';
  position: absolute;
  bottom: -20%;
  right: -15%;
  width: 60%;
  height: 60%;
  background: radial-gradient(circle, rgba(139, 92, 246, 0.08) 0%, transparent 70%);
  pointer-events: none;
  transition: all 0.8s ease;
}

.player-shell.is-playing::before { opacity: 1.4; transform: scale(1.1); }
.player-shell.is-playing::after { opacity: 1.3; transform: scale(1.05); }

/* ========== macOS 原生标题栏占位 ========== */
.titlebar {
  height: 28px;
  flex-shrink: 0;
  width: 100%;
  background: transparent;
  user-select: none;
}

/* ========== 顶部导航栏 ========== */
.header-panel {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.85rem;
  padding: 0.6rem 1.1rem;
  border-radius: var(--radius-lg);
  background: var(--bg-surface);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid var(--border-subtle);
  flex-shrink: 0;
  z-index: 10;
  user-select: none;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 0.85rem;
}

.brand-icon {
  width: 42px;
  height: 42px;
  border-radius: 14px;
  background: linear-gradient(135deg, var(--accent), var(--accent-2));
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.3);
}

.header-title { display: flex; flex-direction: column; gap: 2px; }

.header-title strong {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text-primary);
}

.header-title span {
  color: var(--text-secondary);
  font-size: 0.8rem;
  font-weight: 500;
}

.header-actions { display: flex; flex-wrap: wrap; gap: 0.5rem; }

/* 操作按钮 */
.action-btn {
  border: none;
  padding: 0.6rem 1rem;
  border-radius: 12px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
  letter-spacing: -0.01em;
}

.action-btn svg { flex-shrink: 0; }

.action-btn.primary {
  background: linear-gradient(135deg, var(--accent), var(--accent-2));
  color: #fff;
  box-shadow: 0 4px 20px rgba(99, 102, 241, 0.25);
}

.action-btn.primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(99, 102, 241, 0.35);
}

.action-btn.ghost {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-secondary);
  border: 1px solid var(--border-subtle);
}

.action-btn.ghost:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.action-btn.danger {
  background: rgba(239, 68, 68, 0.12);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.action-btn.danger:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

/* ========== 网格布局 ========== */
.player-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(300px, 0.85fr);
  gap: 1.25rem;
  flex: 1;
  min-height: 0;
}

/* ========== 卡片 ========== */
.player-card,
.playlist-card {
  background: var(--bg-card);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-radius: var(--radius-xl);
  padding: 1.2rem;
  border: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  transition: border-color 0.3s ease;
}

.player-card { justify-content: flex-start; padding-top: 1rem; }

/* ========== 封面区域 ========== */
.cover-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.cover-art-wrapper { position: relative; flex-shrink: 0; cursor: pointer; }

.cover-art {
  width: 200px;
  height: 200px;
  border-radius: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(145deg, #1e1b4b, #312e81);
  box-shadow: 0 16px 48px rgba(99, 102, 241, 0.3);
  position: relative;
  overflow: hidden;
  z-index: 1;
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.cover-art::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, rgba(255,255,255,0.08) 0%, transparent 50%);
  border-radius: 32px;
}

.cover-art-wrapper.empty .cover-art {
  background: rgba(255, 255, 255, 0.06);
  box-shadow: none;
}

.cover-letter {
  font-size: 4.5rem;
  font-weight: 800;
  letter-spacing: -0.04em;
  color: #fff;
  z-index: 2;
  text-shadow: 0 2px 10px rgba(0,0,0,0.2);
}

.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: inherit;
  z-index: 1;
}

.cover-art.has-cover {
  background: none;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
}

.cover-art.has-cover::after {
  display: none;
}

.cover-placeholder {
  color: var(--text-muted);
  z-index: 2;
  opacity: 0.6;
}

.cover-glow {
  position: absolute;
  inset: -8px;
  border-radius: 36px;
  background: conic-gradient(from 0deg, var(--accent), var(--accent-2), var(--accent-3), var(--accent));
  z-index: 0;
  opacity: 0;
  filter: blur(8px);
  transition: opacity 0.5s ease, border-radius 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes glow-rotate { to { transform: rotate(360deg); } }

/* 封面动效：旋转（圆形） */
@keyframes cover-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

/* 封面动效：呼吸 */
@keyframes cover-breathe {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.04); }
}

/* 封面动效：光晕 */
@keyframes cover-glow-pulse {
  0%, 100% { opacity: 0.35; filter: blur(10px); }
  50%      { opacity: 0.7;  filter: blur(16px); }
}

/* 动效触发条件 */

/* 旋转：封面变圆形 + 自转 + 光晕跟随旋转 */
.cover-art-wrapper.effect-rotate.playing .cover-art {
  animation: cover-spin 8s linear infinite;
  border-radius: 50%;
}
.cover-art-wrapper.effect-rotate.playing .cover-art::after {
  border-radius: 50%;
}
.cover-art-wrapper.effect-rotate.playing .cover-glow {
  border-radius: 50%;
  opacity: 0.35;
  animation: glow-rotate 3s linear infinite;
}

/* 呼吸 */
.cover-art-wrapper.effect-pulse.playing .cover-art {
  animation: cover-breathe 2s ease-in-out infinite;
}
.cover-art-wrapper.effect-pulse.playing .cover-glow {
  opacity: 0.3;
  animation: glow-rotate 3s linear infinite;
}

/* 光晕 */
.cover-art-wrapper.effect-glow.playing .cover-glow {
  animation: cover-glow-pulse 2.5s ease-in-out infinite;
}

/* 无动效时播放中的微弱光晕 */
.cover-art-wrapper.effect-none.playing .cover-glow {
  opacity: 0.2;
  animation: glow-rotate 4s linear infinite;
}

/* 暂停/无音乐时全部静止 */
.cover-art-wrapper:not(.playing) .cover-art {
  animation: none;
}
.cover-art-wrapper:not(.playing) .cover-glow {
  animation: none;
  opacity: 0;
}

/* 曲目信息 */
.track-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
  flex: 1;
  text-align: center;
}

.track-name-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  min-width: 0;
  max-width: 100%;
}

.track-name {
  font-size: clamp(1.1rem, 1.6vw, 1.35rem);
  line-height: 1.3;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 260px;
}

.favor-btn {
  flex-shrink: 0;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  border: none;
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.favor-btn:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  transform: scale(1.1);
}

.favor-btn.favored {
  color: #f87171;
  background: rgba(239, 68, 68, 0.12);
}

.track-meta {
  color: var(--text-secondary);
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  flex-wrap: wrap;
}

.meta-badge {
  background: rgba(99, 102, 241, 0.15);
  color: var(--accent-3);
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
}

.meta-dot { color: var(--text-muted); }

/* ========== 播放控制 ========== */
.player-controls { display: flex; flex-direction: column; gap: 0.85rem; }
.progress-section { width: 100%; }

.progress-bar-wrap { display: flex; flex-direction: column; gap: 0.5rem; }

.progress-bar-track { position: relative; width: 100%; height: 6px; }

.progress-bar-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  border-radius: 999px;
  pointer-events: none;
  z-index: 1;
  transition: width 0.1s linear;
}

.seek-input {
  position: absolute;
  top: 50%;
  left: 0;
  width: 100%;
  height: 24px;
  transform: translateY(-50%);
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  cursor: pointer;
  z-index: 2;
  margin: 0;
}

.seek-input::-webkit-slider-runnable-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
}

.seek-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  border: 3px solid var(--accent);
  cursor: pointer;
  margin-top: -5px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  opacity: 0;
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.progress-bar-track:hover .seek-input::-webkit-slider-thumb { opacity: 1; }
.seek-input::-webkit-slider-thumb:active { transform: scale(1.2); }

.time-display {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  font-size: 0.78rem;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
}

.time-divider { opacity: 0.4; }

/* 控制按钮行 */
.control-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

/* 统一控制栏：播放按钮 + 音量 + 快捷键 */
.control-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.playback-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.util-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.ctrl-btn {
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 50%;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.ctrl-btn:hover { background: rgba(255, 255, 255, 0.1); color: var(--text-primary); transform: scale(1.05); }
.ctrl-btn:active { transform: scale(0.95); }
.ctrl-btn.icon-btn { width: 44px; height: 44px; }
.ctrl-btn.icon-btn.small { width: 36px; height: 36px; }
.ctrl-btn.active { background: rgba(99, 102, 241, 0.2); color: var(--accent-3); }

.play-btn-lg {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--accent), var(--accent-2));
  color: #fff;
  box-shadow: 0 8px 30px rgba(99, 102, 241, 0.35);
}

.play-btn-lg:hover {
  transform: scale(1.08);
  box-shadow: 0 12px 40px rgba(99, 102, 241, 0.45);
  color: #fff;
}

.play-btn-lg:active { transform: scale(0.96); }

/* 音量面板 — 内联滑块 */
.volume-inline {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.volume-icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.volume-icon-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
  transform: scale(1.05);
}

.volume-icon-btn:active {
  transform: scale(0.95);
}

.volume-slider-track {
  position: relative;
  width: 90px;
  height: 6px;
  flex-shrink: 0;
}

.volume-slider-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
  border-radius: 999px;
  pointer-events: none;
  z-index: 1;
  transition: width 0.05s linear;
}

.volume-slider-input {
  position: absolute;
  top: 50%;
  left: 0;
  width: 100%;
  height: 28px;
  transform: translateY(-50%);
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  cursor: pointer;
  z-index: 2;
  margin: 0;
}

.volume-slider-input::-webkit-slider-runnable-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
}

.volume-slider-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  border: 2.5px solid var(--accent);
  cursor: pointer;
  margin-top: -4px;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);
  opacity: 0;
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.volume-slider-track:hover .volume-slider-input::-webkit-slider-thumb {
  opacity: 1;
}

.volume-slider-input::-webkit-slider-thumb:active {
  transform: scale(1.2);
}

.volume-pct {
  font-size: 0.7rem;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  min-width: 24px;
  text-align: right;
}
.shortcut-toggle {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.shortcut-toggle:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-secondary);
}

.shortcut-toggle.active {
  background: rgba(99, 102, 241, 0.2);
  color: var(--accent-3);
}

.favor-count {
  font-size: 0.75rem;
  color: #f87171;
  opacity: 0.8;
}

/* 播放列表中的收藏标记 */
.item-favor {
  font-size: 0.7rem;
  flex-shrink: 0;
  opacity: 0.7;
}

.item-name.dimmed { opacity: 0.35; }

/* ========== 歌词 ========== */
.lyrics-section {
  text-align: center;
  padding: 0.5rem 0 0 0;
  min-height: 3.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.3rem;
}

.lyric-line {
  margin: 0;
  font-size: 0.92rem;
  line-height: 1.6;
  color: var(--text-muted);
  transition: all 0.3s ease;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lyric-line.active {
  color: #f1f5f9;
  font-weight: 600;
  font-size: 1rem;
}

.lyric-line.next {
  opacity: 0.4;
  font-size: 0.82rem;
}

/* ========== 波形图 ========== */
.waveform-section {
  padding: 0.25rem 0;
}

/* ========== 音频信息标签行 ========== */
.audio-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding-bottom: 0.35rem;
}

.audio-tag {
  font-size: 0.65rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.35);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 5px;
  padding: 2px 7px;
  white-space: nowrap;
}

.audio-tag.codec {
  color: var(--accent-3);
  background: rgba(99, 102, 241, 0.1);
  border-color: rgba(99, 102, 241, 0.15);
}

.audio-tag.dim {
  opacity: 0.5;
}

/* FX 按钮高亮 */
.fx-btn.active {
  background: rgba(139, 92, 246, 0.2) !important;
  color: var(--accent-3) !important;
}

.fx-mode-tag {
  font-size: 0.7rem;
  font-weight: 700;
  color: var(--accent-3);
  background: rgba(139, 92, 246, 0.15);
  padding: 1px 6px;
  border-radius: 5px;
  white-space: nowrap;
  letter-spacing: -0.01em;
  border: 1px solid rgba(139, 92, 246, 0.2);
}

.inline-fx-tag {
  vertical-align: middle;
}

/* ========== 播放列表 ========== */
.playlist-card { overflow: hidden; }

.hidden-input { position: fixed; left: -9999px; top: 0; }

/* ========== Toast 通知 ========== */
.toast-container {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10000;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  pointer-events: none;
}

.toast-item {
  padding: 0.6rem 1.2rem;
  border-radius: 10px;
  font-size: 0.85rem;
  font-weight: 500;
  pointer-events: auto;
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.toast-info    { background: rgba(99, 102, 241, 0.85); color: #fff; }
.toast-error   { background: rgba(239, 68, 68, 0.85); color: #fff; }
.toast-success { background: rgba(34, 197, 94, 0.85); color: #fff; }

.toast-enter-active { transition: all 0.3s ease; }
.toast-leave-active { transition: all 0.2s ease; }
.toast-enter-from   { opacity: 0; transform: translateY(20px); }
.toast-leave-to     { opacity: 0; transform: translateY(-10px); }

/* ========== Mini 模式 ========== */
.player-shell.is-mini .player-grid {
  grid-template-columns: 1fr;
}
.player-shell.is-mini .playlist-card {
  display: none;
}
.player-shell.is-mini .header-panel {
  justify-content: center;
  padding: 0.4rem 0.8rem;
}
.player-shell.is-mini .header-brand,
.player-shell.is-mini .header-actions .action-btn:not(:first-child) {
  display: none;
}
.player-shell.is-mini .cover-art {
  width: 140px;
  height: 140px;
  border-radius: 20px;
}
.player-shell.is-mini .cover-letter {
  font-size: 3rem;
}
.player-shell.is-mini .player-card {
  padding-top: 0.5rem;
}
.player-shell.is-mini .titlebar {
  height: 12px;
}

/* ========== 响应式 ========== */
@media (max-width: 980px) {
  .player-grid {
    grid-template-columns: 1fr;
    overflow-y: auto;
  }
  .player-card { padding-top: 0.75rem; }
  .cover-art { width: 150px; height: 150px; }
  .cover-letter { font-size: 3.5rem; }
  .playlist-card { max-height: none; flex: 0 0 auto; min-height: 200px; }
  .playlist-item { font-size: 0.85rem; padding: 0.5rem 0.6rem; }
  .item-index { width: 1.6rem; }
}

@media (max-width: 640px) {
  .player-shell { padding: 0; }
  .shell-inner { padding: 6px; gap: 6px; }
  .header-panel { flex-direction: column; align-items: stretch; gap: 0.5rem; padding: 0.75rem 1rem; }
  .header-actions { justify-content: stretch; }
  .action-btn { flex: 1; justify-content: center; font-size: 0.8rem; padding: 0.5rem 0.75rem; }
  .player-card { padding-top: 0.25rem; }
  .cover-art { width: 140px; height: 140px; border-radius: 24px; }
  .cover-letter { font-size: 3rem; }
  .cover-section { gap: 0.5rem; margin-bottom: 0.75rem; }
  .play-btn-lg { width: 48px; height: 48px; }
  .player-card, .playlist-card { padding: 1rem; }
  .playlist-item { padding: 0.4rem 0.5rem; font-size: 0.8rem; gap: 0.4rem; }
  .item-index { width: 1.4rem; font-size: 0.7rem; }
  .item-name { font-size: 0.82rem; }
}
</style>
