// src/lib/stores/editor.ts
import { writable } from 'svelte/store'
import type { PresetName } from '../types'

export interface FontOverrides {
  titleFamily: string | null
  titleSize: number | null
  bodyFamily: string | null
  bodySize: number | null
}

export interface EditorState {
  title: string
  description: string
  articleText: string
  selectedImageSrc: string | null
  fontOverrides: FontOverrides
  preset: PresetName
  customWidth: number
  autoHeight: boolean
  showAttribution: boolean
  summarySource: 'scraped' | 'ai'
  activePrompt: string
  activeTemplateName: string
}

const DEFAULT_STATE: EditorState = {
  title: '',
  description: '',
  articleText: '',
  selectedImageSrc: null,
  fontOverrides: { titleFamily: null, titleSize: null, bodyFamily: null, bodySize: null },
  preset: 'Twitter / X',
  customWidth: 1200,
  autoHeight: false,
  showAttribution: false,
  summarySource: 'scraped',
  activePrompt: '',
  activeTemplateName: '',
}

/** The current state of the editor panel (title, description, fonts, preset, etc.). */
export const editor = writable<EditorState>({ ...DEFAULT_STATE })

/**
 * Reset the editor to its default (empty) state.
 */
export function resetEditor(): void {
  editor.set({ ...DEFAULT_STATE })
}
