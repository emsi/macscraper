<!-- src/lib/components/CardCanvas.svelte -->
<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { appConfig } from '../stores/config'
  import { scraped } from '../stores/scrape'
  import { PLATFORM_PRESETS } from '../types'
  import { renderCard, type CardSpec } from '../canvas/renderer'

  export let theme: 'light' | 'dark'

  let canvas: HTMLCanvasElement
  let overflows = false
  let imageEl: HTMLImageElement | null = null
  let lastImageSrc = ''

  $: preset = PLATFORM_PRESETS.find(p => p.name === $editor.preset) ?? PLATFORM_PRESETS[0]

  $: titleFont = $editor.fontOverrides.titleFamily
    ?? $scraped?.detected_fonts.heading_family
    ?? $appConfig?.style?.title_family
    ?? 'Georgia'

  $: bodyFont = $editor.fontOverrides.bodyFamily
    ?? $scraped?.detected_fonts.body_family
    ?? $appConfig?.style?.body_family
    ?? 'Inter'

  $: titleSize = $editor.fontOverrides.titleSize ?? $appConfig?.style?.title_size ?? 48
  $: bodySize = $editor.fontOverrides.bodySize ?? $appConfig?.style?.body_size ?? 22

  $: domain = (() => {
    try {
      const urlStr = $scraped ? 'https://placeholder.com' : ''
      return urlStr ? new URL(urlStr).hostname : ''
    } catch { return '' }
  })()

  async function loadImage(src: string) {
    if (!src || src === lastImageSrc) return
    lastImageSrc = src
    const dataUrl = src.startsWith('data:') ? src : await invoke<string>('fetch_image', { url: src })
    const img = new Image()
    img.src = dataUrl
    await new Promise<void>(res => { img.onload = () => res() })
    imageEl = img
    draw()
  }

  export function draw() {
    if (!canvas) return
    const spec: CardSpec = {
      image: imageEl,
      title: $editor.title,
      summary: $editor.description,
      domain,
      titleFont,
      titleSize,
      bodyFont,
      bodySize,
      width: $editor.preset === 'Custom' ? $editor.customWidth : preset.width,
      presetHeight: preset.height,
      autoHeight: $editor.autoHeight || preset.height === 0,
      showAttribution: $editor.showAttribution,
      theme,
    }
    overflows = !renderCard(canvas, { ...spec, dpr: window.devicePixelRatio || 1 })
  }

  // Reactive: redraw whenever editor or config changes
  $: { $editor; $appConfig; draw() }

  // Load image when selected image changes
  $: if ($editor.selectedImageSrc) loadImage($editor.selectedImageSrc)

  onMount(draw)

  // Expose canvas for download
  export { canvas }
</script>

<div class="canvas-wrapper">
  {#if overflows && !$editor.autoHeight}
    <span class="overflow-badge">⚠ Text overflows — enable ↕ Auto height</span>
  {/if}
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .canvas-wrapper { position: relative; width: 100%; }
  canvas { width: 100%; height: auto; display: block; border-radius: 4px; }
  .overflow-badge {
    position: absolute; top: 6px; right: 6px;
    background: #e07b39; color: white; font-size: 0.72rem;
    padding: 0.2rem 0.5rem; border-radius: 4px; z-index: 1;
  }
</style>
