import { describe, it, expect } from 'vitest'
import { wrapText, computeCardHeight, LIGHT_COLORS, DARK_COLORS } from './renderer'

// Minimal CanvasRenderingContext2D mock
function makeCtx(avgCharWidth = 10): CanvasRenderingContext2D {
  return {
    measureText: (text: string) => ({ width: text.length * avgCharWidth }),
    font: '',
  } as unknown as CanvasRenderingContext2D
}

describe('wrapText', () => {
  it('returns single line when text fits', () => {
    const ctx = makeCtx(10)
    expect(wrapText(ctx, 'Hello World', 200)).toEqual(['Hello World'])
  })

  it('wraps long text into multiple lines', () => {
    const ctx = makeCtx(10)
    const lines = wrapText(ctx, 'Hello World Test', 100)
    expect(lines.length).toBeGreaterThan(1)
  })

  it('handles empty string', () => {
    const ctx = makeCtx(10)
    expect(wrapText(ctx, '', 200)).toEqual([])
  })
})

describe('computeCardHeight', () => {
  it('returns preset height when autoHeight is false', () => {
    const h = computeCardHeight({
      presetHeight: 628,
      imageHeight: 283,
      titleLines: 1,
      summaryLines: 2,
      titleLineHeight: 60,
      summaryLineHeight: 30,
      padding: 40,
      attributionHeight: 0,
      autoHeight: false,
    })
    expect(h).toBe(628)
  })

  it('returns computed height when autoHeight is true', () => {
    const h = computeCardHeight({
      presetHeight: 628,
      imageHeight: 283,
      titleLines: 2,
      summaryLines: 3,
      titleLineHeight: 60,
      summaryLineHeight: 30,
      padding: 40,
      attributionHeight: 0,
      autoHeight: true,
    })
    // imageHeight + titleLines*titleLH + summaryLines*summaryLH + 3*padding
    expect(h).toBe(283 + 2 * 60 + 3 * 30 + 3 * 40)
  })
})

describe('color themes', () => {
  it('light theme has white background', () => {
    expect(LIGHT_COLORS.background).toBe('#ffffff')
  })

  it('dark theme has dark background', () => {
    expect(DARK_COLORS.background).toBe('#1a1a2e')
  })
})
