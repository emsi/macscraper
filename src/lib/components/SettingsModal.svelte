<!-- src/lib/components/SettingsModal.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { open as openPath } from '@tauri-apps/plugin-shell'
  import { appConfig, apiKey, saveAppConfig } from '../stores/config'

  const dispatch = createEventDispatcher()

  let endpoint = $appConfig?.llm.endpoint ?? 'https://api.openai.com/v1'
  let model = $appConfig?.llm.model ?? 'gpt-4o-mini'
  let key = $apiKey

  async function save() {
    if (!$appConfig) return
    await saveAppConfig({ ...$appConfig, llm: { endpoint, model } })
    await invoke('set_api_key', { key })
    apiKey.set(key)
    dispatch('close')
  }

  async function openConfigFile() {
    const path = await invoke<string>('get_config_path')
    await openPath(path)
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="overlay" role="presentation" on:click|self={() => dispatch('close')}>
  <div class="modal">
    <h2>Settings</h2>

    <label>LLM Endpoint
      <input type="url" bind:value={endpoint} />
    </label>
    <label>Model
      <input type="text" bind:value={model} />
    </label>
    <label>API Key
      <input type="password" bind:value={key} placeholder="sk-… (stored in OS keychain)" />
    </label>

    <p class="note">
      To change the default prompt template or scraping thresholds,
      edit <code>config.toml</code> directly.
    </p>

    <div class="actions">
      <button class="secondary" on:click={openConfigFile}>Open config file</button>
      <button on:click={save}>Save</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0; background: rgba(0,0,0,0.6);
    display: flex; align-items: center; justify-content: center; z-index: 100;
  }
  .modal {
    background: #16162a; border: 1px solid #333; border-radius: 8px;
    padding: 1.5rem; min-width: 360px; display: flex; flex-direction: column; gap: 0.75rem;
  }
  h2 { margin: 0; font-size: 1.1rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; }
  input { width: 100%; }
  .note { font-size: 0.78rem; color: #888; }
  .actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  button.secondary { background: transparent; }
</style>
