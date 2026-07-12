<script setup lang="ts">
import Player from "./components/Player.vue";
</script>

<template>
  <Player />
</template>

<style>
:root {
  font-family: 'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  font-size: 16px;
  line-height: 1.5;
  font-weight: 400;
  color: #f1f5f9;
  background-color: transparent;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

*, *::before, *::after { box-sizing: border-box; }

html, body, #app {
  height: 100%;
  margin: 0;
  overflow: hidden;
  background: transparent;
}

body { min-height: 100vh; background: transparent; }

::selection { background: rgba(99, 102, 241, 0.3); color: #fff; }

::-webkit-scrollbar { width: 4px; height: 4px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: transparent; border-radius: 999px; }
*:hover::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); }
::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.18); }

:focus-visible, button:focus-visible {
  outline: 2px solid rgba(99, 102, 241, 0.5);
  outline-offset: 2px;
}
</style>
