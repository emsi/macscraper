<!-- src/lib/components/ContentSection.svelte -->
<script lang="ts">
  import { scraped } from '../stores/scrape'
  import { editor } from '../stores/editor'
  import ImagePicker from './ImagePicker.svelte'
</script>

<details open>
  <summary>🖼 Content</summary>
  <div class="section-body">
    {#if $scraped}
      <label class="field-label">IMAGE</label>
      <ImagePicker />

      <label class="field-label" for="title-input">TITLE</label>
      <input
        id="title-input"
        type="text"
        value={$editor.title}
        on:input={e => editor.update(s => ({ ...s, title: e.currentTarget.value }))}
      />

      <label class="field-label" for="desc-input">DESCRIPTION</label>
      <textarea
        id="desc-input"
        rows="3"
        value={$editor.description}
        on:input={e => editor.update(s => ({ ...s, description: e.currentTarget.value }))}
      ></textarea>

      <details>
        <summary class="field-label">FULL ARTICLE TEXT (for AI context) ▸</summary>
        <textarea
          rows="5"
          value={$editor.articleText}
          on:input={e => editor.update(s => ({ ...s, articleText: e.currentTarget.value }))}
        ></textarea>
      </details>
    {:else}
      <p class="hint">Scrape a URL to see content.</p>
    {/if}
  </div>
</details>

<style>
  details { border-bottom: 1px solid #333; }
  summary { padding: 0.5rem 0.75rem; cursor: pointer; font-weight: 600; background: #1a1a2e; font-size: 0.85rem; }
  .section-body { padding: 0.6rem 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .field-label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  input, textarea { width: 100%; resize: vertical; }
  .hint { font-size: 0.8rem; color: #666; }
</style>
