<!-- src/lib/components/SummarySection.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { editor } from '../stores/editor'
  import { appConfig, apiKey } from '../stores/config'
  import { substituteTemplate } from '../template'

  let generating = false
  let genError = ''
  let showSaveInput = false
  let newTemplateName = ''

  $: templates = $appConfig?.prompt_templates ?? []

  function selectTemplate(name: string) {
    const tpl = templates.find(t => t.name === name)
    if (tpl) editor.update(e => ({ ...e, activeTemplateName: name, activePrompt: tpl.prompt }))
  }

  async function generate() {
    const state = $editor
    const cfg = $appConfig
    if (!cfg) return
    generating = true
    genError = ''
    try {
      const vars = {
        title: state.title,
        description: state.description,
        article_text: state.articleText,
      }
      const resolvedPrompt = substituteTemplate(state.activePrompt, vars)
      const summary = await invoke<string>('generate_summary', {
        prompt: resolvedPrompt,
        config: { endpoint: cfg.llm.endpoint, model: cfg.llm.model, api_key: $apiKey || null },
      })
      // Write LLM response into the description field (which is what shows on the card)
      editor.update(e => ({ ...e, description: summary }))
    } catch (err) {
      genError = String(err)
    } finally {
      generating = false
    }
  }

  async function saveAsTemplate() {
    if (!newTemplateName.trim() || !$appConfig) return
    const updated = {
      ...$appConfig,
      prompt_templates: [
        ...$appConfig.prompt_templates,
        { name: newTemplateName.trim(), default: false, prompt: $editor.activePrompt },
      ],
    }
    await invoke('save_config', { config: updated })
    appConfig.set(updated)
    showSaveInput = false
    newTemplateName = ''
  }
</script>

<details open>
  <summary>🤖 Summary</summary>
  <div class="section-body">
    <div class="toggle-row">
      <button
        class:active={$editor.summarySource === 'scraped'}
        on:click={() => editor.update(e => ({ ...e, summarySource: 'scraped' }))}
      >Scraped</button>
      <button
        class:active={$editor.summarySource === 'ai'}
        on:click={() => editor.update(e => ({ ...e, summarySource: 'ai' }))}
      >AI Generate</button>
    </div>

    {#if $editor.summarySource === 'ai'}
      <label class="field-label" for="template-select">PROMPT TEMPLATE</label>
      <select id="template-select" value={$editor.activeTemplateName} on:change={e => selectTemplate(e.currentTarget.value)}>
        {#each templates as tpl}
          <option value={tpl.name}>{tpl.name}</option>
        {/each}
      </select>

      <textarea
        rows="4"
        value={$editor.activePrompt}
        on:input={e => editor.update(s => ({ ...s, activePrompt: e.currentTarget.value }))}
      ></textarea>

      <div class="row">
        <button on:click={generate} disabled={generating}>
          {generating ? 'Generating…' : 'Generate ✨'}
        </button>
        <button class="secondary" on:click={() => showSaveInput = !showSaveInput}>
          Save as template…
        </button>
      </div>

      {#if showSaveInput}
        <div class="save-row">
          <input type="text" placeholder="Template name" bind:value={newTemplateName} />
          <button on:click={saveAsTemplate} disabled={!newTemplateName.trim()}>Save</button>
        </div>
      {/if}

      {#if genError}
        <p class="error">{genError}</p>
      {/if}
    {/if}
  </div>
</details>

<style>
  details { border-bottom: 1px solid #333; }
  summary { padding: 0.5rem 0.75rem; cursor: pointer; font-weight: 600; background: #1a1a2e; font-size: 0.85rem; }
  .section-body { padding: 0.6rem 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
  .field-label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  .toggle-row, .row { display: flex; gap: 0.4rem; }
  .save-row { display: flex; gap: 0.4rem; }
  .save-row input { flex: 1; }
  button.active { background: #4a9eff; color: white; border-color: #4a9eff; }
  button.secondary { background: transparent; }
  textarea, select { width: 100%; resize: vertical; }
  .error { color: #e07b39; font-size: 0.8rem; }
</style>
