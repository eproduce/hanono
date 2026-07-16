<script setup lang="ts">
interface Track {
  name: string;
  url: string;
  path?: string;
  persistent?: boolean;
}

interface HistoryEntry { name: string; time: number; }

interface FilteredItem {
  track: Track;
  idx: number;
  time?: number;
}

const props = defineProps<{
  playlist: Track[];
  favorites: Track[];
  playHistory: HistoryEntry[];
  currentIndex: number;
  isPlaying: boolean;
  playlistFilter: 'all' | 'favorites' | 'history';
  filteredPlaylist: FilteredItem[];
}>();

const emit = defineEmits<{
  'update:playlistFilter': [value: 'all' | 'favorites' | 'history'];
  selectTrack: [index: number];
  selectFavoritesTrack: [favIndex: number];
  removeTrack: [index: number];
  removeFromFavorites: [favIndex: number];
  removeHistory: [index: number];
  clearHistory: [];
  contextMenu: [event: MouseEvent, index: number];
}>();

function isFavorited(track: Track): boolean {
  return props.favorites.some(f => f.path === track.path || f.url === track.url);
}

function formatHistoryTime(ts: number): string {
  const d = new Date(ts);
  const now = new Date();
  const diffMin = Math.floor((now.getTime() - d.getTime()) / 60000);
  if (diffMin < 1) return '刚刚';
  if (diffMin < 60) return `${diffMin}分钟前`;
  const diffH = Math.floor(diffMin / 60);
  if (diffH < 24) return `${diffH}小时前`;
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
}

function onItemClick(fi: number, item: FilteredItem) {
  if (props.playlistFilter === 'favorites') {
    emit('selectFavoritesTrack', fi);
  } else if (props.playlistFilter === 'history') {
    if (item.idx >= 0) emit('selectTrack', item.idx);
  } else {
    emit('selectTrack', item.idx);
  }
}

function onRemove(fi: number) {
  if (props.playlistFilter === 'history') {
    emit('removeHistory', fi);
  } else if (props.playlistFilter === 'favorites') {
    emit('removeFromFavorites', fi);
  } else {
    emit('removeTrack', fi);
  }
}
</script>

<template>
  <aside class="playlist-card">
    <div class="playlist-header">
      <div class="playlist-tabs">
        <button class="tab-btn" :class="{ active: playlistFilter === 'all' }" @click="emit('update:playlistFilter', 'all')">
          全部<span class="tab-count">{{ playlist.length }}</span>
        </button>
        <button class="tab-btn" :class="{ active: playlistFilter === 'favorites' }" @click="emit('update:playlistFilter', 'favorites')">
          ❤️ 收藏<span class="tab-count">{{ favorites.length }}</span>
        </button>
        <button class="tab-btn" :class="{ active: playlistFilter === 'history' }" @click="emit('update:playlistFilter', 'history')">
          🕐 历史<span class="tab-count">{{ playHistory.length }}</span>
        </button>
      </div>
      <button v-if="playlistFilter === 'history' && playHistory.length > 0"
        class="clear-history-btn" @click="emit('clearHistory')">清空历史</button>
    </div>

    <ul class="playlist-list" v-if="filteredPlaylist.length > 0">
      <li v-for="(item, fi) in filteredPlaylist" :key="item.track.url" :class="{ active: item.idx === currentIndex }">
        <div class="playlist-row" @contextmenu="emit('contextMenu', $event, item.idx)">
          <button class="playlist-item" @click="onItemClick(fi, item)">
            <span class="item-index">
              <template v-if="item.idx === currentIndex && isPlaying">
                <span class="eq-bars"><i></i><i></i><i></i><i></i></span>
              </template>
              <template v-else-if="playlistFilter === 'history' && item.time">
                {{ formatHistoryTime(item.time) }}
              </template>
              <template v-else>{{ playlistFilter === 'favorites' ? fi + 1 : item.idx + 1 }}</template>
            </span>
            <span class="item-name" :class="{ dimmed: playlistFilter === 'history' && item.idx < 0 }">{{ item.track.name }}</span>
            <span v-if="isFavorited(item.track)" class="item-favor">❤️</span>
          </button>
          <button class="remove-btn" @click.stop="onRemove(fi)" title="移除">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none"><path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/></svg>
          </button>
        </div>
      </li>
    </ul>

    <div v-else class="empty-state">
      <div class="empty-icon">
        <svg v-if="playlistFilter === 'favorites'" width="56" height="56" viewBox="0 0 24 24" fill="none"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/></svg>
        <svg v-else-if="playlistFilter === 'history'" width="56" height="56" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.2"/><path d="M12 6v6l4 2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
        <svg v-else width="56" height="56" viewBox="0 0 24 24" fill="none"><path d="M9 18V5l12-2v13" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/><circle cx="6" cy="18" r="2.5" stroke="currentColor" stroke-width="1.2"/><circle cx="18" cy="16" r="2.5" stroke="currentColor" stroke-width="1.2"/></svg>
      </div>
      <p class="empty-title">
        {{ playlistFilter === 'favorites' ? '暂无收藏' : playlistFilter === 'history' ? '暂无记录' : '列表为空' }}
      </p>
      <p class="empty-desc">
        <template v-if="playlistFilter === 'favorites'">点击曲目旁的 ♡ 即可收藏</template>
        <template v-else-if="playlistFilter === 'history'">播放过的曲目会出现在这里</template>
        <template v-else>拖拽音频文件到此处<br>或点击「导入音频」开始</template>
      </p>
    </div>
  </aside>
</template>

<style scoped>
.playlist-card {
  background: rgba(255, 255, 255, 0.07);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-radius: 28px;
  padding: 1.5rem;
  border: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.playlist-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
  flex-shrink: 0;
}

.playlist-tabs { display: flex; gap: 4px; }

.tab-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  padding: 0.4rem 0.75rem;
  font-size: 0.8rem;
  font-weight: 500;
  color: #94a3b8;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.18s ease;
  display: flex; align-items: center; gap: 5px;
}
.tab-btn:hover { color: #e2e8f0; background: rgba(255, 255, 255, 0.08); }
.tab-btn.active { background: rgba(99, 102, 241, 0.15); color: #c4b5fd; border-color: rgba(99, 102, 241, 0.3); }

.tab-count {
  font-size: 0.65rem; font-weight: 700; color: #64748b;
  background: rgba(255, 255, 255, 0.06); padding: 1px 6px; border-radius: 999px;
}

.clear-history-btn {
  border: none; padding: 0.35rem 0.65rem; font-size: 0.75rem; border-radius: 10px;
  cursor: pointer; font-family: inherit; font-weight: 600; flex-shrink: 0;
  background: rgba(239, 68, 68, 0.12); color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.2); transition: all 0.2s ease;
}
.clear-history-btn:hover { background: rgba(239, 68, 68, 0.2); color: #fca5a5; }

.playlist-list {
  list-style: none; margin: 0; padding: 0;
  overflow-y: auto; flex: 1; min-height: 0;
}
.playlist-list::-webkit-scrollbar { width: 4px; }
.playlist-list::-webkit-scrollbar-thumb { background: transparent; border-radius: 999px; }
.playlist-list:hover::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); }

.playlist-row {
  display: flex; align-items: center; gap: 4px;
  padding: 2px 6px; border-radius: 10px; transition: background 0.15s ease;
}
.playlist-row:hover { background: rgba(255, 255, 255, 0.04); }
li.active .playlist-row { background: rgba(99, 102, 241, 0.1); }

.playlist-item {
  flex: 1; display: flex; align-items: center; gap: 0.65rem;
  background: none; border: none; color: inherit; cursor: pointer;
  font-family: inherit; padding: 0.5rem 4px; text-align: left; min-width: 0;
}

.item-index {
  width: 32px; text-align: center; font-size: 0.7rem; font-weight: 600;
  color: #64748b; flex-shrink: 0;
}
li.active .item-index { color: #a78bfa; }

.eq-bars { display: flex; align-items: flex-end; gap: 1.5px; height: 12px; justify-content: center; }
.eq-bars i {
  width: 2.5px; background: #a78bfa; border-radius: 1px;
  animation: eq-jump 0.6s ease-in-out infinite alternate;
}
.eq-bars i:nth-child(1) { height: 6px; }
.eq-bars i:nth-child(2) { height: 10px; animation-delay: 0.15s; }
.eq-bars i:nth-child(3) { height: 8px; animation-delay: 0.3s; }
.eq-bars i:nth-child(4) { height: 11px; animation-delay: 0.45s; }

@keyframes eq-jump { to { transform: scaleY(0.3); } }

.item-name {
  font-size: 0.85rem; font-weight: 500; color: #e2e8f0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.item-name.dimmed { color: #64748b; }
li.active .item-name { color: #f1f5f9; font-weight: 600; }

.item-favor { font-size: 0.75rem; flex-shrink: 0; }

.remove-btn {
  background: none; border: none; color: #64748b; cursor: pointer;
  padding: 4px; border-radius: 6px; flex-shrink: 0;
  opacity: 0; transition: all 0.15s ease;
}
.playlist-row:hover .remove-btn { opacity: 1; }
.remove-btn:hover { color: #f87171; background: rgba(239, 68, 68, 0.1); }

.empty-state {
  flex: 1; display: flex; flex-direction: column;
  align-items: center; justify-content: center; gap: 0.75rem;
  color: #475569; padding: 2rem 1rem;
}
.empty-title { font-size: 0.95rem; font-weight: 600; color: #64748b; margin: 0; }
.empty-desc { font-size: 0.8rem; color: #475569; margin: 0; text-align: center; line-height: 1.5; }
</style>
