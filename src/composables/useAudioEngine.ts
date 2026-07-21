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

  function initAudioContext() {
    if (audioCtx) return;
    audioCtx = new AudioContext();
    sourceNode = audioCtx.createMediaElementSource(audio);

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
    masterGain.gain.value = 1;

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

  function ensureFxReady(): boolean {
    if (!audioCtx) initAudioContext();
    if (audioCtx && audioCtx.state === 'suspended') audioCtx.resume();
    return audioCtx !== null && eqNodes.length === 5;
  }

  function ensureAudioContext() {
    if (!audioCtx) initAudioContext();
    if (audioCtx?.state === 'suspended') audioCtx.resume();
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
    ensureFxReady();
    showFxPanel.value = true;
  }

  /** 临时静音输出（音源切换用） */
  function muteForSwitch() {
    if (!audioCtx) initAudioContext();
    if (audioCtx && masterGain) {
      masterGain.gain.setValueAtTime(0, audioCtx.currentTime);
    }
  }

  /** 恢复输出音量 */
  function unmuteAfterSwitch(v: number) {
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
