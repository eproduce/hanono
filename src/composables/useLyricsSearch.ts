import { invoke } from '@tauri-apps/api/core';

export interface LyricLine {
  time: number;
  text: string;
}

/** Minimal track info needed for lyrics search */
export interface TrackLike {
  name: string;
  path?: string;
  url: string;
}

/** LRC 时间戳：兼容 [mm:ss] / [mm:ss.xx] / [mm:ss.xxx] / [mm:ss:xx] */
const LRC_TIME_RE = /\[(\d{1,3}):(\d{1,2})(?:[.:](\d{1,3}))?\]\s*(.*)/;

/** 解析单行 LRC，返回时间与文本；不是歌词行则返回 null */
function parseLrcLine(line: string): { time: number; text: string } | null {
  const m = line.match(LRC_TIME_RE);
  if (!m) return null;
  const min = parseInt(m[1], 10);
  const sec = parseInt(m[2], 10);
  const frac = m[3] ? parseInt(m[3], 10) / Math.pow(10, m[3].length) : 0;
  return { time: min * 60 + sec + frac, text: m[4].trim() };
}

/**
 * Parse LRC format text into timed lyric lines.
 */
export function parseLrc(lrcText: string): LyricLine[] {
  const lines: LyricLine[] = [];
  for (const line of lrcText.split('\n')) {
    const parsed = parseLrcLine(line);
    if (parsed && parsed.text) lines.push(parsed);
  }
  return lines.sort((a, b) => a.time - b.time);
}

/**
 * Parse LRC and also extract hanono offset header: [hanono:offset:1.5]
 * Returns the parsed lines and any stored offset value.
 */
export function parseLrcWithOffset(lrcText: string): { lines: LyricLine[]; offset: number } {
  let offset = 0;
  const lines: LyricLine[] = [];
  const offsetRegex = /^\[hanono:offset:([-]?\d+\.?\d*)\]/i;

  for (const line of lrcText.split('\n')) {
    // Check for offset header
    const om = line.match(offsetRegex);
    if (om) {
      offset = parseFloat(om[1]);
      continue;
    }
    const parsed = parseLrcLine(line);
    if (parsed && parsed.text) lines.push(parsed);
  }
  return { lines: lines.sort((a, b) => a.time - b.time), offset };
}

/** 方括号/方头括号内容，多为 [FLAC] [HQ] 这类音质标注 */
const BRACKET_RE = /\[[^\]]*\]|【[^】]*】/g;
/** 含明确“版本/音质”关键词的圆括注，如 (Official MV) (Live) (无损) */
const JUNK_PAREN_RE =
  /[（(][^）)]*(official|mv|m\/v|lyric|remaster|live|hd|hq|flac|320k|128k|无损|高音质|完整版|现场版|翻自|cover|demo|inst|伴奏|纯音乐|超清|高清)[^）)]*[）)]/gi;
/** 末尾音质后缀，如 "xxx HQ"、"xxx 320K" */
const JUNK_TAIL_RE = /\s*[-_ ](hq|hd|320k|128k|flac|ape|wav|无损|高音质|完整版)\s*$/gi;
/** 文件名开头的音轨号，如 "01." / "01 - " / "1_ " */
const TRACK_NO_RE = /^\s*\d{1,3}\s*[.\-_、，,]+\s*/;

/** 去掉文件名中的噪声（音轨号、音质标注、版本括注等） */
function stripJunk(raw: string): string {
  return raw
    .replace(BRACKET_RE, ' ')
    .replace(JUNK_PAREN_RE, ' ')
    .replace(JUNK_TAIL_RE, ' ')
    .replace(TRACK_NO_RE, ' ')
    .replace(/_+/g, ' ')
    .replace(/\s{2,}/g, ' ')
    .trim();
}

/**
 * Attempt to parse artist and title from a track filename.
 * Common pattern: "Artist - Title.ext"（先去除音轨号/音质标注等噪声）
 */
export function parseArtistTitle(trackName: string): { artist: string; title: string } {
  const name = stripJunk(trackName.replace(/\.[^.]+$/, ''));
  const separators = [' - ', ' – ', ' — ', '－', '-', '–', '—'];
  for (const sep of separators) {
    const idx = name.indexOf(sep);
    if (idx > 0) {
      const artist = name.substring(0, idx).trim();
      const title = name.substring(idx + sep.length).trim();
      if (artist && title) return { artist, title };
    }
  }
  return { artist: '', title: name.trim() };
}

/** 匹配用归一化：小写、去 feat 之后内容、只保留字母数字（兼容中日韩） */
export function normalizeForMatch(raw: string): string {
  return raw
    .toLowerCase()
    .replace(BRACKET_RE, ' ')
    .replace(JUNK_PAREN_RE, ' ')
    .replace(/\b(?:feat|ft|featuring)\b\.?\s+.*$/i, ' ')
    .replace(/[^\p{L}\p{N}]+/gu, '')
    .trim();
}

function charBigrams(s: string): Set<string> {
  const out = new Set<string>();
  if (s.length === 0) return out;
  if (s.length === 1) {
    out.add(s);
    return out;
  }
  for (let i = 0; i < s.length - 1; i++) out.add(s.slice(i, i + 2));
  return out;
}

/** 0~1 文本相似度：完全相同 = 1，包含关系 = 0.92，否则用二元组 Dice 系数 */
export function textSimilarity(a: string, b: string): number {
  const x = normalizeForMatch(a);
  const y = normalizeForMatch(b);
  if (!x || !y) return 0;
  if (x === y) return 1;
  if (x.length >= 2 && y.length >= 2 && (x.includes(y) || y.includes(x))) return 0.92;
  const A = charBigrams(x);
  const B = charBigrams(y);
  let inter = 0;
  for (const g of A) if (B.has(g)) inter++;
  return (2 * inter) / (A.size + B.size);
}

/** LrcAPI jsonapi 返回的候选项 */
export interface LyricCandidate {
  title?: string;
  artist?: string;
  album?: string;
  lyrics?: string;
  id?: string;
  cover?: string;
}

/** 接受阈值：低于此分数就当作“不是这首歌”，宁可不显示也不错配 */
export const MIN_TITLE_SCORE = 0.62;
export const MIN_MATCH_SCORE = 0.45;

/**
 * 从候选中挑出最匹配当前曲目的一条。
 * 评分方式与 LrcAPI 服务端一致：以标题为主、歌手为辅。
 */
export function pickBestCandidate(
  candidates: LyricCandidate[],
  artist: string,
  title: string,
): { candidate: LyricCandidate; score: number; titleScore: number } | null {
  let best: { candidate: LyricCandidate; score: number; titleScore: number } | null = null;

  for (const c of candidates) {
    if (!c?.lyrics || !c.lyrics.trim()) continue;
    const titleScore = textSimilarity(title, c.title ?? '');
    const artistScore = artist ? textSimilarity(artist, c.artist ?? '') : 0;
    const score = Math.sqrt((titleScore * (artistScore + 1)) / 2);
    if (!best || score > best.score) best = { candidate: c, score, titleScore };
  }

  if (!best) return null;
  if (best.titleScore < MIN_TITLE_SCORE || best.score < MIN_MATCH_SCORE) return null;
  return best;
}

/** 读取 LRC 头部的 [ti:]/[ar:] 元信息 */
function readLrcTags(lrc: string): { ti?: string; ar?: string } {
  const tags: { ti?: string; ar?: string } = {};
  for (const line of lrc.split('\n').slice(0, 40)) {
    const m = line.match(/^\[(ti|ar|al):(.*)\]\s*$/i);
    if (!m) continue;
    const key = m[1].toLowerCase();
    const val = m[2].trim();
    if (!val) continue;
    if (key === 'ti') tags.ti = val;
    else if (key === 'ar') tags.ar = val;
  }
  return tags;
}

/** 用 LRC 自带元信息兜底校验：标题对不上、或歌手明确冲突时丢弃 */
function isLrcConsistent(lrc: string, artist: string, title: string): boolean {
  const { ti, ar } = readLrcTags(lrc);
  if (!ti) return true; // 无元信息，无法判断
  if (textSimilarity(title, ti) < MIN_TITLE_SCORE) return false;
  if (ar && artist && textSimilarity(artist, ar) < 0.3) return false;
  return true;
}

/** 退回单曲接口（旧接口），并用 LRC 元信息尽量校验 */
async function searchOnlineLyricsSingle(artist: string, title: string): Promise<LyricLine[]> {
  try {
    const lrcText = await invoke<string | null>('lrcapi_search_lyrics', { artist, title });
    if (!lrcText || !lrcText.trim()) return [];
    const lines = parseLrc(lrcText);
    if (lines.length >= 2 && isLrcConsistent(lrcText, artist, title)) {
      console.log('[lrcapi] lyrics found via backend:', lrcText.length, 'bytes');
      return lines;
    }
    console.warn('[lrcapi] 单曲接口返回结果与曲目信息不符，已丢弃');
  } catch (e) {
    console.warn('[lrcapi] backend lyrics failed:', e);
  }
  return [];
}

/**
 * Search for lyrics online via Tauri backend (LrcAPI proxy, no CORS issues).
 *
 * 关键：先拿“候选列表（带 title/artist）”再做匹配校验，只有确实匹配当前曲目才采用，
 * 否则宁可返回空，避免出现“搜出来一个完全不对的歌词”。
 */
export async function searchOnlineLyrics(track: TrackLike): Promise<LyricLine[]> {
  const { artist, title } = parseArtistTitle(track.name);
  if (!title) return [];

  // Step 1: 候选 + 校验
  try {
    const candidates = await invoke<LyricCandidate[]>('lrcapi_search_lyrics_candidates', {
      artist,
      title,
      limit: 6,
    });

    if (Array.isArray(candidates)) {
      const best = pickBestCandidate(candidates, artist, title);
      if (best) {
        console.log(
          `[lyrics] 匹配成功 score=${best.score.toFixed(2)} → ${best.candidate.artist || '?'} - ${best.candidate.title || '?'}`,
        );
        const lines = parseLrc(best.candidate.lyrics ?? '');
        if (lines.length >= 2) return lines;
      } else if (candidates.length > 0) {
        console.warn(
          '[lyrics] 候选均与当前曲目不匹配，已丢弃:',
          candidates.map(c => `${c.artist || '?'} - ${c.title || '?'}`).join(' | '),
        );
      }
      // 候选接口可用时不再退回旧接口，避免又拿回不可信的歌词
      return [];
    }
  } catch (e) {
    console.warn('[lrcapi] candidates failed:', e);
  }

  // Step 2: 候选接口不可用时，退回单曲接口（附带元信息校验）
  return searchOnlineLyricsSingle(artist, title);
}

// ========== Cover API ==========

/**
 * Search for cover art via LrcAPI.
 * Returns cover image URL or null.
 */
/**
 * Search for cover art via Tauri backend (LrcAPI proxy).
 * Returns cover image URL or null.
 */
export async function searchOnlineCover(
  artist: string,
  title: string,
): Promise<string | null> {
  try {
    const url = await invoke<string | null>('lrcapi_search_cover', { artist, title });
    if (url) {
      console.log('[lrcapi] cover found via backend:', url);
      return url;
    }
  } catch (e) {
    console.warn('[lrcapi] backend cover failed:', e);
  }
  return null;
}
