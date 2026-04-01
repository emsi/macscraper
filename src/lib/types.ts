// src/lib/types.ts
export interface ScrapedData {
  title: string
  description: string
  og_image: string | null
  all_images: ImageMeta[]
  article_text: string
  detected_fonts: DetectedFonts
}

export interface ImageMeta {
  src: string
  alt: string
  width: number | null
  height: number | null
}

export interface DetectedFonts {
  heading_family: string | null
  body_family: string | null
  google_fonts_url: string | null
}

export interface AppConfig {
  llm: LlmConfig
  scraping: ScrapingConfig
  style?: StyleConfig
  prompt_templates: PromptTemplate[]
}

export interface LlmConfig {
  endpoint: string
  model: string
}

export interface ScrapingConfig {
  article_min_chars: number
  image_min_width: number
  max_stylesheets: number
}

export interface StyleConfig {
  title_family: string
  title_size: number
  body_family: string
  body_size: number
}

export interface PromptTemplate {
  name: string
  default?: boolean
  prompt: string
}

export interface LlmCallConfig {
  endpoint: string
  model: string
  api_key: string | null
}

export const PLATFORM_PRESETS = [
  { name: 'Twitter / X', width: 1200, height: 628 },
  { name: 'Facebook / OG', width: 1200, height: 630 },
  { name: 'LinkedIn', width: 1200, height: 627 },
  { name: 'Instagram Square', width: 1080, height: 1080 },
  { name: 'Instagram Portrait', width: 1080, height: 1350 },
  { name: 'Custom', width: 1200, height: 0 }, // height 0 = auto
] as const

export type PresetName = typeof PLATFORM_PRESETS[number]['name']
