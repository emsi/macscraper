<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from 'svelte'
  import { loadAppConfig } from './lib/stores/config'
  import UrlBar from './lib/components/UrlBar.svelte'
  import ContentSection from './lib/components/ContentSection.svelte'
  import SummarySection from './lib/components/SummarySection.svelte'
  import StyleSection from './lib/components/StyleSection.svelte'
  import PreviewPanel from './lib/components/PreviewPanel.svelte'
  import SettingsModal from './lib/components/SettingsModal.svelte'

  let showSettings = false

  onMount(() => loadAppConfig())
</script>

<div class="app">
  <header class="titlebar">
    <span class="app-name">macscraper</span>
    <button class="settings-btn" on:click={() => showSettings = true}>⚙</button>
  </header>

  <div class="workspace">
    <!-- Left: editor panel -->
    <div class="editor-panel">
      <UrlBar />
      <div class="accordion">
        <ContentSection />
        <SummarySection />
        <StyleSection />
      </div>
    </div>

    <!-- Right: live preview -->
    <div class="preview-panel">
      <PreviewPanel />
    </div>
  </div>
</div>

{#if showSettings}
  <SettingsModal on:close={() => showSettings = false} />
{/if}

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) { margin: 0; font-family: system-ui, sans-serif; background: #0f0f1a; color: #e0e0e0; }
  :global(input, textarea, select, button) {
    font-family: inherit; font-size: inherit;
    background: #1e1e2e; color: #e0e0e0; border: 1px solid #333; border-radius: 4px;
  }
  :global(button) { cursor: pointer; padding: 0.3rem 0.75rem; }
  :global(button:disabled) { opacity: 0.5; cursor: default; }

  .app { display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
  .titlebar {
    display: flex; justify-content: space-between; align-items: center;
    padding: 0.4rem 0.75rem; background: #16162a; border-bottom: 1px solid #333;
  }
  .app-name { font-weight: 600; }
  .settings-btn { background: none; border: none; font-size: 1.1rem; }
  .workspace { display: flex; flex: 1; overflow: hidden; }
  .editor-panel { width: 42%; border-right: 1px solid #333; display: flex; flex-direction: column; overflow: hidden; }
  .accordion { flex: 1; overflow-y: auto; }
  .preview-panel { flex: 1; overflow-y: auto; }
</style>
