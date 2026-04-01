// src/lib/stores/scrape.ts
import { writable } from 'svelte/store'
import type { ScrapedData } from '../types'

/** The most recently scraped page data, or null before first scrape. */
export const scraped = writable<ScrapedData | null>(null)

/** True while a scrape is in progress. */
export const scraping = writable(false)

/** The most recent scrape error message, or null if no error. */
export const scrapeError = writable<string | null>(null)
