<!-- src/App.svelte -->
<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { loadAppConfig } from './lib/stores/config'
  import UrlBar from './lib/components/UrlBar.svelte'
  import ContentSection from './lib/components/ContentSection.svelte'
  import SummarySection from './lib/components/SummarySection.svelte'
  import StyleSection from './lib/components/StyleSection.svelte'
  import PreviewPanel from './lib/components/PreviewPanel.svelte'
  import SettingsModal from './lib/components/SettingsModal.svelte'

  let showSettings = false
  let editorWidth = 42   // percentage of workspace width
  let dragging = false
  let workspace: HTMLDivElement

  onMount(async () => {
    loadAppConfig()
    const saved = await invoke<number | null>('get_split_ratio').catch(() => null)
    if (saved !== null && saved !== undefined) editorWidth = saved
  })

  function onDividerMouseDown(e: MouseEvent) {
    dragging = true
    e.preventDefault()
  }

  function onMouseMove(e: MouseEvent) {
    if (!dragging || !workspace) return
    const rect = workspace.getBoundingClientRect()
    const x = e.clientX - rect.left
    editorWidth = Math.max(20, Math.min(70, (x / rect.width) * 100))
  }

  async function onMouseUp() {
    if (!dragging) return
    dragging = false
    await invoke('save_split_ratio', { ratio: editorWidth }).catch(() => {})
  }
</script>

<div class="app">
  <header class="titlebar">
    <span class="app-name">macscraper</span>
    <button class="settings-btn" on:click={() => showSettings = true}>⚙</button>
  </header>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="workspace"
    class:dragging
    bind:this={workspace}
    on:mousemove={onMouseMove}
    on:mouseup={onMouseUp}
    on:mouseleave={onMouseUp}
  >
    <!-- Left: editor panel -->
    <div class="editor-panel" style="width: {editorWidth}%">
      <UrlBar />
      <div class="accordion">
        <ContentSection />
        <SummarySection />
        <StyleSection />
      </div>
    </div>

    <!-- Drag handle -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="divider" on:mousedown={onDividerMouseDown}></div>

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
  .workspace.dragging { cursor: col-resize; user-select: none; }
  .editor-panel { flex-shrink: 0; display: flex; flex-direction: column; overflow: hidden; }
  .divider {
    width: 5px; flex-shrink: 0; background: #2a2a3e; cursor: col-resize;
    transition: background 0.15s;
  }
  .divider:hover, .workspace.dragging .divider { background: #4a9eff; }
  .accordion { flex: 1; overflow-y: auto; }
  .preview-panel { flex: 1; overflow-y: auto; min-width: 0; }
</style>
