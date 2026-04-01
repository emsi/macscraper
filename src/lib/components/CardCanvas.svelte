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

  // canvas: visible display element (sized to its container, no CSS scaling)
  // renderCanvas: offscreen full-res canvas used for download/copy
  let canvas: HTMLCanvasElement
  const renderCanvas = document.createElement('canvas')

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

  /** Build the card spec from current editor state. */
  function buildSpec(): CardSpec {
    return {
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
  }

  export function draw() {
    if (!canvas) return
    const spec = buildSpec()
    const dpr = window.devicePixelRatio || 1

    // Render full-res card to the offscreen canvas (used for download/copy too)
    overflows = !renderCard(renderCanvas, spec)

    // Scale down to the display canvas using Canvas drawImage with
    // imageSmoothingQuality='high' — this uses a proper box/bicubic filter,
    // unlike CSS scaling of canvas elements in WebKitGTK which is low quality.
    const displayW = canvas.offsetWidth || spec.width
    const aspectRatio = renderCanvas.height / renderCanvas.width
    const displayH = Math.round(displayW * aspectRatio)

    canvas.width = Math.round(displayW * dpr)
    canvas.height = Math.round(displayH * dpr)

    const ctx = canvas.getContext('2d')!
    ctx.imageSmoothingEnabled = true
    ctx.imageSmoothingQuality = 'high'
    ctx.scale(dpr, dpr)
    ctx.drawImage(renderCanvas, 0, 0, displayW, displayH)
  }

  /** Save card as PNG via native file-save dialog (at full preset dimensions). */
  export async function triggerDownload(filename: string): Promise<void> {
    const offscreen = document.createElement('canvas')
    renderCard(offscreen, buildSpec())
    const base64 = offscreen.toDataURL('image/png').replace(/^data:image\/png;base64,/, '')
    await invoke('save_png', { data: base64, suggestedName: filename })
  }

  /** Copy card PNG to the system clipboard (at full preset dimensions). */
  export async function copyToClipboard(): Promise<void> {
    const offscreen = document.createElement('canvas')
    renderCard(offscreen, buildSpec())
    const base64 = offscreen.toDataURL('image/png').replace(/^data:image\/png;base64,/, '')
    await invoke('copy_png_to_clipboard', { data: base64 })
  }

  // Reactive: redraw whenever editor or config changes
  $: { $editor; $appConfig; draw() }

  // Load image when selected image changes
  $: if ($editor.selectedImageSrc) loadImage($editor.selectedImageSrc)

  onMount(() => {
    draw()
    // Redraw at the correct buffer size whenever the panel is resized or
    // the window goes fullscreen — prevents the old buffer being CSS-upscaled.
    const observer = new ResizeObserver(() => draw())
    observer.observe(canvas)
    return () => observer.disconnect()
  })
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
