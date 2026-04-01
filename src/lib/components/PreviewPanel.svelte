<!-- src/lib/components/PreviewPanel.svelte -->
<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { PLATFORM_PRESETS } from '../types'
  import CardCanvas from './CardCanvas.svelte'

  $: preset = PLATFORM_PRESETS.find(p => p.name === $editor.preset) ?? PLATFORM_PRESETS[0]
  $: dims = $editor.preset === 'Custom'
    ? `${$editor.customWidth}×auto`
    : `${preset.width}×${$editor.autoHeight ? 'auto' : preset.height}`

  let lightCanvas: CardCanvas
  let darkCanvas: CardCanvas

  let copyError = ''
  let activeTheme: 'light' | 'dark' = 'light'

  onMount(async () => {
    const saved = await invoke<string | null>('get_active_theme').catch(() => null)
    if (saved === 'light' || saved === 'dark') activeTheme = saved
  })

  async function switchTheme(theme: 'light' | 'dark') {
    activeTheme = theme
    await invoke('save_active_theme', { theme }).catch(() => {})
    // Trigger redraw on newly-visible canvas (offsetWidth was 0 while hidden)
    await tick()
    if (theme === 'light') lightCanvas?.draw()
    else darkCanvas?.draw()
  }

  async function download(theme: 'light' | 'dark') {
    const ref = theme === 'light' ? lightCanvas : darkCanvas
    await ref.triggerDownload(`card-${theme}.png`)
  }

  async function copy(theme: 'light' | 'dark') {
    copyError = ''
    try {
      const ref = theme === 'light' ? lightCanvas : darkCanvas
      await ref.copyToClipboard()
    } catch (e) {
      copyError = String(e)
    }
  }
</script>

<div class="preview">
  <div class="preview-header">
    Live preview · {$editor.preset} ({dims}) · <span class="dot">auto-updating</span>
  </div>
  <div class="tabs">
    <button class="tab" class:active={activeTheme === 'light'} on:click={() => switchTheme('light')}>☀ Light</button>
    <button class="tab" class:active={activeTheme === 'dark'} on:click={() => switchTheme('dark')}>🌑 Dark</button>
  </div>
  <div class="cards">
    <div class="card-block" class:hidden={activeTheme !== 'light'}>
      <CardCanvas bind:this={lightCanvas} theme="light" />
      <div class="btn-row">
        <button on:click={() => download('light')}>⬇ Download PNG</button>
        <button class="secondary" on:click={() => copy('light')}>⎘ Copy</button>
      </div>
    </div>
    <div class="card-block" class:hidden={activeTheme !== 'dark'}>
      <CardCanvas bind:this={darkCanvas} theme="dark" />
      <div class="btn-row">
        <button on:click={() => download('dark')}>⬇ Download PNG</button>
        <button class="secondary" on:click={() => copy('dark')}>⎘ Copy</button>
      </div>
    </div>
    {#if copyError}
      <p class="copy-error">{copyError}</p>
    {/if}
  </div>
</div>

<style>
  .preview { display: flex; flex-direction: column; height: 100%; }
  .preview-header { padding: 0.5rem 0.75rem; font-size: 0.75rem; color: #888; background: #1a1a2e; border-bottom: 1px solid #333; }
  .dot { color: #4a9eff; }
  .tabs {
    display: flex; background: #16162a; border-bottom: 1px solid #333; padding: 0 0.5rem;
  }
  .tab {
    background: none; border: none; border-bottom: 2px solid transparent;
    border-radius: 0; padding: 0.4rem 0.75rem; color: #888; font-size: 0.82rem;
    cursor: pointer; transition: color 0.15s, border-color 0.15s;
  }
  .tab:hover { color: #e0e0e0; }
  .tab.active { color: #e0e0e0; border-bottom-color: #4a9eff; }
  .cards { flex: 1; overflow-y: auto; padding: 0.75rem; display: flex; flex-direction: column; gap: 1rem; }
  .card-block { display: flex; flex-direction: column; gap: 0.3rem; }
  .card-block.hidden { display: none; }
  .btn-row { display: flex; gap: 0.4rem; margin-top: 0.2rem; }
  button { font-size: 0.82rem; }
  button.secondary { background: transparent; }
  .copy-error { font-size: 0.82rem; color: #e07b39; }
</style>
