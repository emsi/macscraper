import { describe, it, expect } from 'vitest'
import { buildFontList, injectGoogleFonts } from './fonts'

describe('buildFontList', () => {
  it('puts detected fonts first with detected marker', () => {
    const list = buildFontList('Playfair Display', 'Source Sans Pro')
    expect(list[0].value).toBe('Playfair Display')
    expect(list[0].label).toContain('✦')
    expect(list[1].value).toBe('Source Sans Pro')
    expect(list[1].label).toContain('✦')
  })

  it('includes curated Google Fonts after detected', () => {
    const list = buildFontList(null, null)
    const values = list.map(f => f.value)
    expect(values).toContain('Inter')
    expect(values).toContain('Roboto')
    expect(values).toContain('Merriweather')
  })

  it('includes generic fallbacks at the end', () => {
    const list = buildFontList(null, null)
    const last = list[list.length - 1].value
    expect(['serif', 'sans-serif', 'monospace']).toContain(last)
  })

  it('deduplicates when detected font matches curated font', () => {
    const list = buildFontList('Inter', null)
    const interCount = list.filter(f => f.value === 'Inter').length
    expect(interCount).toBe(1)
  })
})
