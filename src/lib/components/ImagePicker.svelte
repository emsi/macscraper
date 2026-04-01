<!-- src/lib/components/ImagePicker.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import { editor } from '../stores/editor'
  import { scraped } from '../stores/scrape'

  $: images = $scraped?.all_images ?? []
  $: selected = $editor.selectedImageSrc

  function selectImage(src: string) {
    editor.update(e => ({ ...e, selectedImageSrc: src }))
  }

  async function pickFromDisk() {
    const file = await open({
      multiple: false,
      filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif'] }]
    })
    if (typeof file === 'string') {
      // Fetch via Rust to get a data URL (works for local file:// URLs too)
      const dataUrl = await invoke<string>('fetch_image', { url: `file://${file}` })
      editor.update(e => ({ ...e, selectedImageSrc: dataUrl }))
    }
  }
</script>

<div class="image-row">
  {#each images as img}
    <button
      class="thumb"
      class:selected={img.src === selected}
      on:click={() => selectImage(img.src)}
      title={img.alt}
    >
      <img src={img.src} alt={img.alt} loading="lazy" />
    </button>
  {/each}
  <button class="thumb disk-tile" on:click={pickFromDisk} title="Upload from disk">
    +disk
  </button>
</div>

<style>
  .image-row { display: flex; gap: 0.3rem; flex-wrap: wrap; padding: 0.25rem 0; }
  .thumb { width: 56px; height: 40px; padding: 0; border: 2px solid transparent; border-radius: 3px; overflow: hidden; }
  .thumb.selected { border-color: #4a9eff; }
  .thumb img { width: 100%; height: 100%; object-fit: cover; }
  .disk-tile { background: #252535; border-style: dashed; font-size: 0.65rem; color: #888; }
</style>
