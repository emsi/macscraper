<!-- src/lib/components/PreviewPanel.svelte -->
<script lang="ts">
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
  <div class="cards">
    <div class="card-block">
      <span class="label">☀️ White</span>
      <CardCanvas bind:this={lightCanvas} theme="light" />
      <div class="btn-row">
        <button on:click={() => download('light')}>⬇ Download PNG</button>
        <button class="secondary" on:click={() => copy('light')}>⎘ Copy</button>
      </div>
    </div>
    <div class="card-block">
      <span class="label">🌑 Dark</span>
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
  .cards { flex: 1; overflow-y: auto; padding: 0.75rem; display: flex; flex-direction: column; gap: 1rem; }
  .card-block { display: flex; flex-direction: column; gap: 0.3rem; }
  .label { font-size: 0.75rem; color: #888; }
  .btn-row { display: flex; gap: 0.4rem; margin-top: 0.2rem; }
  button { font-size: 0.75rem; }
  button.secondary { background: transparent; }
  .copy-error { font-size: 0.75rem; color: #e07b39; }
</style>
