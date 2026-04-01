<!-- src/lib/components/UrlBar.svelte -->
<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { scraped, scraping, scrapeError } from '../stores/scrape'
  import { editor } from '../stores/editor'
  import { appConfig } from '../stores/config'
  import { injectGoogleFonts } from '../fonts'
  import type { ScrapedData } from '../types'

  let url = ''

  onMount(async () => {
    const last = await invoke<string | null>('get_last_url')
    if (last) url = last
  })

  async function handleScrape() {
    if (!url.trim()) return
    scraping.set(true)
    scrapeError.set(null)
    try {
      await invoke('save_last_url', { url })
      const cfg = $appConfig
      const scrapingConfig = cfg?.scraping ?? {
        article_min_chars: 200, image_min_width: 300, max_stylesheets: 5,
      }
      const data = await invoke<ScrapedData>('scrape_url', { url, scrapingConfig })
      scraped.set(data)

      // Inject Google Fonts if detected
      if (data.detected_fonts.google_fonts_url) {
        injectGoogleFonts(data.detected_fonts.google_fonts_url)
      }

      // Pre-populate editor with scraped data
      const defaultTemplate = cfg?.prompt_templates.find(t => t.default)
        ?? cfg?.prompt_templates[0]
      editor.update(e => ({
        ...e,
        title: data.title,
        description: data.description,
        articleText: data.article_text,
        selectedImageSrc: data.og_image
          ?? data.all_images.find(i => (i.width ?? 0) >= scrapingConfig.image_min_width)?.src
          ?? data.all_images[0]?.src
          ?? null,
        fontOverrides: {
          titleFamily: data.detected_fonts.heading_family,
          titleSize: null,
          bodyFamily: data.detected_fonts.body_family,
          bodySize: null,
        },
        activePrompt: defaultTemplate?.prompt ?? '',
        activeTemplateName: defaultTemplate?.name ?? '',
      }))
    } catch (err) {
      scrapeError.set(String(err))
    } finally {
      scraping.set(false)
    }
  }
</script>

<div class="url-bar">
  <input
    type="url"
    bind:value={url}
    placeholder="https://myblog.com/my-article"
    disabled={$scraping}
    on:keydown={e => e.key === 'Enter' && handleScrape()}
  />
  <button on:click={handleScrape} disabled={$scraping || !url.trim()}>
    {$scraping ? 'Scraping…' : 'Scrape'}
  </button>
</div>

{#if $scrapeError}
  <p class="error">{$scrapeError}</p>
{/if}

<style>
  .url-bar { display: flex; gap: 0.5rem; padding: 0.6rem 0.75rem; }
  input { flex: 1; }
  .error { color: var(--color-error, #e07b39); font-size: 0.8rem; padding: 0 0.75rem; }
</style>
