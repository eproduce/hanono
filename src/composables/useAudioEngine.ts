import { ref, computed, type Ref } from 'vue';

// ========== 音效系统 (Web Audio API) ==========

export type EqPresetKey = 'flat' | 'pop' | 'rock' | 'jazz' | 'classical' | 'vocal' | 'bass';

export interface EqBand {
  name: string;
  freq: number;
  gain: Ref<number>;
  q: number;
}

const eqPresets: Record<EqPresetKey, number[]> = {
  flat:      [ 0,  0,  0,  0,  0],
  pop:       [-1,  2,  3,  2,  1],
  rock:      [ 3,  0, -2,  1,  2],
  jazz:      [ 2,  1,  0, -1,  0],
  classical: [ 1,  0,  0,  1,  2],
  vocal:     [-2, -1,  3,  2,  1],
  bass:      [ 8,  4,  0, -1, -2],
};

export const presetLabels: Record<EqPresetKey, string> = {
  flat: '默认', pop: '流行', rock: '摇滚', jazz: '爵士',
  classical: '古典', vocal: '人声', bass: '低音',
};

export function useAudioEngine(audio: HTMLAudioElement) {
  let audioCtx: AudioContext | null = null;
  let sourceNode: MediaElementAudioSourceNode | null = null;
  let bassBoostNode: BiquadFilterNode | null = null;
  let eqNodes: BiquadFilterNode[] = [];
  let surroundNode: StereoPannerNode | null = null;
  let reverbNode: ConvolverNode | null = null;
  let wetGainNode: GainNode | null = null;
  let dryGainNode: GainNode | null = null;
  let masterGain: GainNode | null = null;

  // —— 变调相关状态 ——
  // WebKit（macOS WKWebView / Safari）下，AudioContext 的采样率与音频文件不一致时，
  // 媒体元素输出的重采样会让音高、速度发生变化（听感即“变调”）。
  // 因此这里记录音频文件采样率，并在建立管线时按该采样率创建 AudioContext。
  let sampleRateHint: number | null = null;
  let sampleRateResolved = false; // 已拿到（或确认拿不到）文件采样率
  let graphWanted = false;      // 已开始播放 / 需要 Web Audio 管线
  let graphUnavailable = false; // 管线建立失败（只能原生播放）
  let desiredMasterGain = 1;    // 期望主增益（音量 × 响度归一化）

  const eqBands: EqBand[] = [
    { name: '60Hz',  freq: 60,   gain: ref<number>(0),  q: 0.8 },
    { name: '250Hz', freq: 250,  gain: ref<number>(0),  q: 0.8 },
    { name: '1kHz',  freq: 1000, gain: ref<number>(0),  q: 0.8 },
    { name: '4kHz',  freq: 4000, gain: ref<number>(0),  q: 0.8 },
    { name: '8kHz',  freq: 8000, gain: ref<number>(0),  q: 0.8 },
  ];

  const bassBoost = ref(0);
  const surroundAmount = ref(0);
  const reverbAmount = ref(0);
  const currentPreset = ref<EqPresetKey>('flat');
  const showFxPanel = ref(false);

  const currentFxLabel = computed(() => {
    const effectsOn = bassBoost.value > 0 || surroundAmount.value > 0 || reverbAmount.value > 0;
    if (currentPreset.value !== 'flat') return presetLabels[currentPreset.value];
    if (effectsOn) return '自定义';
    return '';
  });

  /** 创建 AudioContext；优先使用与音频文件一致的采样率，避免音高/速度变化 */
  function createAudioContext(rate: number | null): AudioContext | null {
    const Ctor: typeof AudioContext | undefined =
      window.AudioContext ??
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!Ctor) return null;

    if (rate && rate >= 8000 && rate <= 384000) {
      try {
        return new Ctor({ sampleRate: rate });
      } catch {
        /* 该采样率不被支持，回退到默认采样率 */
      }
    }
    try {
      return new Ctor();
    } catch (e) {
      console.warn('[fx] AudioContext 创建失败:', e);
      return null;
    }
  }

  function initAudioContext() {
    if (audioCtx || graphUnavailable) return;
    const ctx = createAudioContext(sampleRateHint);
    if (!ctx) {
      graphUnavailable = true;
      return;
    }

    let src: MediaElementAudioSourceNode;
    try {
      // 同一个 <audio> 只能创建一次 MediaElementSource，失败后彻底退回原生播放
      src = ctx.createMediaElementSource(audio);
    } catch (e) {
      console.warn('[fx] createMediaElementSource 失败，改用原生播放:', e);
      graphUnavailable = true;
      void ctx.close().catch(() => {});
      return;
    }

    audioCtx = ctx;
    sourceNode = src;

    bassBoostNode = audioCtx.createBiquadFilter();
    bassBoostNode.type = 'lowshelf';
    bassBoostNode.frequency.value = 80;
    bassBoostNode.gain.value = bassBoost.value;

    eqNodes = eqBands.map(band => {
      const filter = audioCtx!.createBiquadFilter();
      filter.type = 'peaking';
      filter.frequency.value = band.freq;
      filter.Q.value = band.q;
      filter.gain.value = band.gain.value;
      return filter;
    });

    surroundNode = audioCtx.createStereoPanner();
    surroundNode.pan.value = 0;

    reverbNode = audioCtx.createConvolver();
    reverbNode.buffer = createReverbBuffer(audioCtx);

    wetGainNode = audioCtx.createGain();
    wetGainNode.gain.value = 0;
    dryGainNode = audioCtx.createGain();
    dryGainNode.gain.value = 1;
    masterGain = audioCtx.createGain();
    // 用“期望增益”初始化，避免管线建立瞬间音量跳变
    masterGain.gain.value = desiredMasterGain;

    let prev: AudioNode = sourceNode;
    prev.connect(bassBoostNode!);
    prev = bassBoostNode!;
    for (const eq of eqNodes) {
      prev.connect(eq);
      prev = eq;
    }
    prev.connect(surroundNode!);
    prev = surroundNode!;

    prev.connect(dryGainNode!);
    prev.connect(reverbNode!);
    reverbNode!.connect(wetGainNode!);

    dryGainNode!.connect(masterGain!);
    wetGainNode!.connect(masterGain!);
    masterGain!.connect(audioCtx.destination);

    if (audioCtx.state === 'suspended') void audioCtx.resume().catch(() => {});
    console.log(
      '[fx] AudioContext',
      audioCtx.sampleRate + 'Hz',
      sampleRateHint ? `(文件 ${sampleRateHint}Hz)` : '(未知文件采样率)',
    );
  }

  function createReverbBuffer(ctx: AudioContext): AudioBuffer {
    const sampleRate = ctx.sampleRate;
    const length = sampleRate * 1.5;
    const buffer = ctx.createBuffer(2, length, sampleRate);
    for (let ch = 0; ch < 2; ch++) {
      const data = buffer.getChannelData(ch);
      for (let i = 0; i < length; i++) {
        data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (sampleRate * 0.3));
      }
    }
    return buffer;
  }

  /** 音效操作时建立管线（用户显式操作，必须保证功能可用，不等采样率） */
  function ensureFxReady(): boolean {
    graphWanted = true;
    if (!audioCtx) initAudioContext();
    if (audioCtx && audioCtx.state === 'suspended') void audioCtx.resume().catch(() => {});
    return audioCtx !== null && eqNodes.length === 5;
  }

  /**
   * 播放时标记需要 Web Audio 管线。
   * 已知采样率才建立 → 保证 AudioContext 采样率与音频文件一致（修复变调）；
   * 未知时先记下意图，等 setSampleRateHint() 拿到采样率后再建立。
   */
  function ensureAudioContext() {
    graphWanted = true;
    if (!audioCtx && sampleRateResolved) initAudioContext();
    if (audioCtx?.state === 'suspended') void audioCtx.resume().catch(() => {});
  }

  /** 记录当前音频文件的采样率（由播放器在读取音频信息后调用） */
  function setSampleRateHint(rate: number | null | undefined) {
    if (!rate || !Number.isFinite(rate) || rate <= 0) return;
    sampleRateHint = Math.round(rate);
    sampleRateResolved = true;
    if (audioCtx && Math.abs(audioCtx.sampleRate - sampleRateHint) > 1) {
      console.warn(
        `[fx] 采样率不一致：AudioContext ${audioCtx.sampleRate}Hz / 文件 ${sampleRateHint}Hz，可能出现音高变化`,
      );
    }
    if (graphWanted && !audioCtx) initAudioContext();
  }

  /** 拿不到采样率（非 Tauri / 读取失败）时的兜底：按默认采样率建立管线，保证音效与响度归一化仍可用 */
  function markSampleRateUnknown() {
    sampleRateResolved = true;
    if (graphWanted && !audioCtx) initAudioContext();
  }

  function applyPreset(key: EqPresetKey) {
    currentPreset.value = key;
    const gains = eqPresets[key];
    eqBands.forEach((band, i) => { band.gain.value = gains[i]; });
    if (ensureFxReady()) {
      eqBands.forEach((band, i) => {
        eqNodes[i].gain.setTargetAtTime(band.gain.value, audioCtx!.currentTime, 0.02);
      });
    }
    if (key === 'bass') {
      bassBoost.value = 6;
      if (ensureFxReady()) bassBoostNode!.gain.setTargetAtTime(6, audioCtx!.currentTime, 0.02);
    } else if (bassBoost.value === 6) {
      bassBoost.value = 0;
      if (ensureFxReady()) bassBoostNode!.gain.setTargetAtTime(0, audioCtx!.currentTime, 0.02);
    }
  }

  function updateBassBoost() {
    if (!ensureFxReady()) return;
    bassBoostNode!.gain.setTargetAtTime(bassBoost.value, audioCtx!.currentTime, 0.02);
  }

  function updateEqBand(index: number) {
    if (!ensureFxReady()) return;
    eqNodes[index].gain.setTargetAtTime(eqBands[index].gain.value, audioCtx!.currentTime, 0.02);
    currentPreset.value = 'flat';
  }

  function updateSurround() {
    if (!ensureFxReady()) return;
    surroundNode!.pan.setTargetAtTime(surroundAmount.value * 0.8, audioCtx!.currentTime, 0.02);
  }

  function updateReverb() {
    if (!ensureFxReady()) return;
    wetGainNode!.gain.setTargetAtTime(reverbAmount.value * 0.5, audioCtx!.currentTime, 0.02);
    dryGainNode!.gain.setTargetAtTime(1 - reverbAmount.value * 0.3, audioCtx!.currentTime, 0.02);
  }

  function resetAllEffects() {
    bassBoost.value = 0;
    surroundAmount.value = 0;
    reverbAmount.value = 0;
    applyPreset('flat');
    if (!ensureFxReady()) return;
    bassBoostNode!.gain.setTargetAtTime(0, audioCtx!.currentTime, 0.02);
    surroundNode!.pan.setTargetAtTime(0, audioCtx!.currentTime, 0.02);
    wetGainNode!.gain.setTargetAtTime(0, audioCtx!.currentTime, 0.02);
    dryGainNode!.gain.setTargetAtTime(1, audioCtx!.currentTime, 0.02);
  }

  function setMasterVolume(v: number) {
    desiredMasterGain = v;
    if (masterGain && audioCtx) {
      masterGain.gain.setTargetAtTime(v, audioCtx.currentTime, 0.02);
    }
  }

  // Event handlers for sliders
  function onBassBoostInput(e: Event) {
    bassBoost.value = parseFloat((e.target as HTMLInputElement).value);
    updateBassBoost();
  }
  function onSurroundInput(e: Event) {
    surroundAmount.value = parseFloat((e.target as HTMLInputElement).value);
    updateSurround();
  }
  function onReverbInput(e: Event) {
    reverbAmount.value = parseFloat((e.target as HTMLInputElement).value);
    updateReverb();
  }
  function onEqBandInput(index: number, e: Event) {
    eqBands[index].gain.value = parseFloat((e.target as HTMLInputElement).value);
    updateEqBand(index);
  }

  function openFxPanel() {
    // 只打开面板，不建立管线；用户真正调节音效时才建立
    showFxPanel.value = true;
  }

  /** 临时静音输出（音源切换用）。管线未建立时无需处理（音量由 audio.volume 兜底） */
  function muteForSwitch() {
    if (!audioCtx || !masterGain) return;
    // 强制 resume 确保 currentTime 在推进
    if (audioCtx.state === 'suspended') void audioCtx.resume().catch(() => {});
    masterGain.gain.cancelScheduledValues(audioCtx.currentTime);
    masterGain.gain.setValueAtTime(0, audioCtx.currentTime);
  }

  /** 恢复输出音量 */
  function unmuteAfterSwitch(v: number) {
    desiredMasterGain = v;
    if (audioCtx && masterGain) {
      masterGain.gain.setTargetAtTime(v, audioCtx.currentTime, 0.05);
    }
  }

  return {
    eqBands,
    bassBoost,
    surroundAmount,
    reverbAmount,
    currentPreset,
    currentFxLabel,
    showFxPanel,
    eqPresets,
    ensureAudioContext,
    ensureFxReady,
    setSampleRateHint,
    markSampleRateUnknown,
    applyPreset,
    updateBassBoost,
    updateEqBand,
    updateSurround,
    updateReverb,
    resetAllEffects,
    setMasterVolume,
    onBassBoostInput,
    onSurroundInput,
    onReverbInput,
    onEqBandInput,
    openFxPanel,
    muteForSwitch,
    unmuteAfterSwitch,
  };
}
