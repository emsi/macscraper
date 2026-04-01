<!-- src/lib/components/StyleSection.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { appConfig } from '../stores/config'
  import { scraped } from '../stores/scrape'
  import { buildFontList } from '../fonts'
  import { PLATFORM_PRESETS, type PresetName } from '../types'

  $: detectedHeading = $scraped?.detected_fonts.heading_family ?? null
  $: detectedBody = $scraped?.detected_fonts.body_family ?? null
  $: fontList = buildFontList(detectedHeading, detectedBody)

  $: titleFont = $editor.fontOverrides.titleFamily ?? detectedHeading ?? 'Georgia'
  $: bodyFont = $editor.fontOverrides.bodyFamily ?? detectedBody ?? 'Inter'
  $: titleSize = $editor.fontOverrides.titleSize ?? $appConfig?.style?.title_size ?? 48
  $: bodySize = $editor.fontOverrides.bodySize ?? $appConfig?.style?.body_size ?? 22

  async function saveStyleDefault() {
    if (!$appConfig) return
    const updated = {
      ...$appConfig,
      style: {
        title_family: titleFont,
        title_size: titleSize,
        body_family: bodyFont,
        body_size: bodySize,
      },
    }
    await invoke('save_config', { config: updated })
    appConfig.set(updated)
  }
</script>

<details>
  <summary>🎨 Style</summary>
  <div class="section-body">
    <div class="font-grid">
      <span class="field-label">TITLE FONT</span>
      <span class="field-label">SIZE</span>
      <select
        value={titleFont}
        on:change={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, titleFamily: e.currentTarget.value } }))}
      >
        {#each fontList as f}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
      <input
        type="number" min="12" max="120"
        value={titleSize}
        on:input={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, titleSize: +e.currentTarget.value } }))}
      />

      <span class="field-label">BODY FONT</span>
      <span class="field-label">SIZE</span>
      <select
        value={bodyFont}
        on:change={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, bodyFamily: e.currentTarget.value } }))}
      >
        {#each fontList as f}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
      <input
        type="number" min="10" max="80"
        value={bodySize}
        on:input={e => editor.update(s => ({ ...s, fontOverrides: { ...s.fontOverrides, bodySize: +e.currentTarget.value } }))}
      />
    </div>

    <label class="field-label">PLATFORM PRESET</label>
    <select
      value={$editor.preset}
      on:change={e => editor.update(s => ({ ...s, preset: e.currentTarget.value as PresetName }))}
    >
      {#each PLATFORM_PRESETS as p}
        <option value={p.name}>{p.name} ({p.width}×{p.height || 'auto'})</option>
      {/each}
    </select>

    {#if $editor.preset === 'Custom'}
      <label class="field-label">CUSTOM WIDTH (px)</label>
      <input
        type="number" min="400" max="3000"
        value={$editor.customWidth}
        on:input={e => editor.update(s => ({ ...s, customWidth: +e.currentTarget.value }))}
      />
    {/if}

    <label class="checkbox-label">
      <input
        type="checkbox"
        checked={$editor.autoHeight}
        on:change={e => editor.update(s => ({ ...s, autoHeight: e.currentTarget.checked }))}
      />
      ↕ Auto height
    </label>

    <label class="checkbox-label">
      <input
        type="checkbox"
        checked={$editor.showAttribution}
        on:change={e => editor.update(s => ({ ...s, showAttribution: e.currentTarget.checked }))}
      />
      Show source attribution
    </label>

    <button on:click={saveStyleDefault}>Save style as default</button>
  </div>
</details>

<style>
  details { border-bottom: 1px solid #333; }
  summary { padding: 0.5rem 0.75rem; cursor: pointer; font-weight: 600; background: #1a1a2e; font-size: 0.85rem; }
  .section-body { padding: 0.6rem 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .field-label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  .font-grid { display: grid; grid-template-columns: 1fr 56px; gap: 0.3rem; align-items: center; }
  select { width: 100%; }
  .checkbox-label { display: flex; align-items: center; gap: 0.4rem; font-size: 0.85rem; }
</style>
