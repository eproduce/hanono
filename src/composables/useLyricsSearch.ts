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

/**
 * Parse LRC format text into timed lyric lines.
 */
export function parseLrc(lrcText: string): LyricLine[] {
  const lines: LyricLine[] = [];
  // Support [mm:ss.xx] and [mm:ss.xxx]
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

/**
 * Parse LRC and also extract hanono offset header: [hanono:offset:1.5]
 * Returns the parsed lines and any stored offset value.
 */
export function parseLrcWithOffset(lrcText: string): { lines: LyricLine[]; offset: number } {
  let offset = 0;
  const lines: LyricLine[] = [];
  const regex = /\[(\d{2}):(\d{2}(?:\.\d{2,3})?)\](.*)/;
  const offsetRegex = /^\[hanono:offset:([-]?\d+\.?\d*)\]/i;

  for (const line of lrcText.split('\n')) {
    // Check for offset header
    const om = line.match(offsetRegex);
    if (om) {
      offset = parseFloat(om[1]);
      continue;
    }
    const m = line.match(regex);
    if (m) {
      const min = parseInt(m[1]);
      const sec = parseFloat(m[2]);
      const text = m[3].trim();
      if (text) lines.push({ time: min * 60 + sec, text });
    }
  }
  return { lines: lines.sort((a, b) => a.time - b.time), offset };
}

/**
 * Attempt to parse artist and title from a track filename.
 * Common pattern: "Artist - Title.ext"
 */
export function parseArtistTitle(trackName: string): { artist: string; title: string } {
  const name = trackName.replace(/\.[^.]+$/, '');
  const separators = [' - ', ' – ', ' — ', '-', '–', '—'];
  for (const sep of separators) {
    const idx = name.indexOf(sep);
    if (idx > 0) {
      return {
        artist: name.substring(0, idx).trim(),
        title: name.substring(idx + sep.length).trim(),
      };
    }
  }
  return { artist: '', title: name.trim() };
}

/**
 * Search for lyrics online via Tauri backend (LrcAPI proxy, no CORS issues).
 * Returns parsed LyricLine array or empty array.
 */
export async function searchOnlineLyrics(track: TrackLike): Promise<LyricLine[]> {
  const { artist, title } = parseArtistTitle(track.name);
  if (!title) return [];

  try {
    const lrcText = await invoke<string | null>('lrcapi_search_lyrics', { artist, title });
    if (lrcText && lrcText.trim()) {
      console.log('[lrcapi] lyrics found via backend:', lrcText.length, 'bytes');
      return parseLrc(lrcText);
    }
  } catch (e) {
    console.warn('[lrcapi] backend lyrics failed:', e);
  }

  return [];
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
