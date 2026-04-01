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

  function download(theme: 'light' | 'dark') {
    const ref = theme === 'light' ? lightCanvas : darkCanvas
    // Access the exposed canvas element
    const cvs: HTMLCanvasElement = (ref as any)?.canvas
    if (!cvs) return
    cvs.toBlob(blob => {
      if (!blob) return
      const a = document.createElement('a')
      a.href = URL.createObjectURL(blob)
      a.download = `card-${theme}-${Date.now()}.png`
      a.click()
      URL.revokeObjectURL(a.href)
    }, 'image/png')
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
      <button on:click={() => download('light')}>⬇ Download PNG</button>
    </div>
    <div class="card-block">
      <span class="label">🌑 Dark</span>
      <CardCanvas bind:this={darkCanvas} theme="dark" />
      <button on:click={() => download('dark')}>⬇ Download PNG</button>
    </div>
  </div>
</div>

<style>
  .preview { display: flex; flex-direction: column; height: 100%; }
  .preview-header { padding: 0.5rem 0.75rem; font-size: 0.75rem; color: #888; background: #1a1a2e; border-bottom: 1px solid #333; }
  .dot { color: #4a9eff; }
  .cards { flex: 1; overflow-y: auto; padding: 0.75rem; display: flex; flex-direction: column; gap: 1rem; }
  .card-block { display: flex; flex-direction: column; gap: 0.3rem; }
  .label { font-size: 0.75rem; color: #888; }
  button { align-self: flex-start; font-size: 0.75rem; margin-top: 0.2rem; }
</style>
