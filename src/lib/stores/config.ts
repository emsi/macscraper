// src/lib/stores/config.ts
import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types'

/** The loaded application configuration, or null before config is loaded. */
export const appConfig = writable<AppConfig | null>(null)

/** The current API key (loaded from OS keychain, never persisted to disk directly). */
export const apiKey = writable<string>('')

/**
 * Load the application config from config.toml and the API key from OS keychain.
 * Should be called once on app startup.
 */
export async function loadAppConfig(): Promise<void> {
  const config = await invoke<AppConfig>('load_config')
  appConfig.set(config)
  const key = await invoke<string | null>('get_api_key')
  apiKey.set(key ?? '')
}

/**
 * Save the application config to config.toml and update the store.
 *
 * @param config - The updated config to save.
 */
export async function saveAppConfig(config: AppConfig): Promise<void> {
  await invoke('save_config', { config })
  appConfig.set(config)
}
