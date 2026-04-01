// src/lib/canvas/renderer.ts

export interface CardColors {
  background: string
  title: string
  body: string
  attribution: string
  separator: string
}

export const LIGHT_COLORS: CardColors = {
  background: '#ffffff',
  title: '#111111',
  body: '#444444',
  attribution: '#aaaaaa',
  separator: '#eeeeee',
}

export const DARK_COLORS: CardColors = {
  background: '#1a1a2e',
  title: '#f0f0f0',
  body: '#cccccc',
  attribution: '#555555',
  separator: '#2a2a3e',
}

export interface CardSpec {
  image: HTMLImageElement | null
  title: string
  summary: string
  domain: string
  titleFont: string
  titleSize: number
  bodyFont: string
  bodySize: number
  width: number
  presetHeight: number  // 0 = auto
  autoHeight: boolean
  showAttribution: boolean
  theme: 'light' | 'dark'
}

/**
 * Split text into lines that fit within maxWidth pixels.
 *
 * :param ctx: The canvas rendering context (used for text measurement).
 * :param text: The text to wrap.
 * :param maxWidth: Maximum line width in pixels.
 * :return: Array of wrapped lines.
 */
export function wrapText(
  ctx: CanvasRenderingContext2D,
  text: string,
  maxWidth: number
): string[] {
  if (!text) return []
  const words = text.split(' ')
  const lines: string[] = []
  let current = ''
  for (const word of words) {
    const candidate = current ? `${current} ${word}` : word
    if (ctx.measureText(candidate).width > maxWidth && current) {
      lines.push(current)
      current = word
    } else {
      current = candidate
    }
  }
  if (current) lines.push(current)
  return lines
}

export interface HeightParams {
  presetHeight: number
  imageHeight: number
  titleLines: number
  summaryLines: number
  titleLineHeight: number
  summaryLineHeight: number
  padding: number
  attributionHeight: number
  autoHeight: boolean
}

/**
 * Calculate final card height.
 * If autoHeight is true or presetHeight is 0, compute from content dimensions.
 *
 * :param p: The height parameters.
 * :return: The final card height in pixels.
 */
export function computeCardHeight(p: HeightParams): number {
  const contentHeight =
    p.imageHeight +
    p.titleLines * p.titleLineHeight +
    p.summaryLines * p.summaryLineHeight +
    3 * p.padding +
    p.attributionHeight
  if (p.autoHeight || p.presetHeight === 0) return contentHeight
  return p.presetHeight
}

/**
 * Render a social media card onto a canvas element.
 * Returns true if content fits within the preset height, false if it overflows.
 *
 * :param canvas: The target canvas element.
 * :param spec: The card specification (content, fonts, dimensions, theme).
 * :return: Whether the content fits within the preset height.
 */
export function renderCard(canvas: HTMLCanvasElement, spec: CardSpec): boolean {
  const colors = spec.theme === 'dark' ? DARK_COLORS : LIGHT_COLORS
  const ctx = canvas.getContext('2d')!
  const PAD = 48
  const IMAGE_RATIO = 0.45
  const TITLE_LINE_HEIGHT = Math.round(spec.titleSize * 1.3)
  const BODY_LINE_HEIGHT = Math.round(spec.bodySize * 1.5)
  const ATTR_HEIGHT = spec.showAttribution ? 32 : 0
  const textWidth = spec.width - PAD * 2

  // Measure text
  ctx.font = `bold ${spec.titleSize}px "${spec.titleFont}", serif`
  const titleLines = wrapText(ctx, spec.title, textWidth)

  ctx.font = `${spec.bodySize}px "${spec.bodyFont}", sans-serif`
  const summaryLines = wrapText(ctx, spec.summary, textWidth)

  const imageHeight = Math.round(spec.width * IMAGE_RATIO)
  const contentHeight = computeCardHeight({
    presetHeight: spec.presetHeight,
    imageHeight,
    titleLines: titleLines.length,
    summaryLines: summaryLines.length,
    titleLineHeight: TITLE_LINE_HEIGHT,
    summaryLineHeight: BODY_LINE_HEIGHT,
    padding: PAD,
    attributionHeight: ATTR_HEIGHT,
    autoHeight: spec.autoHeight,
  })

  const fits = spec.presetHeight === 0 || spec.autoHeight ||
    contentHeight <= spec.presetHeight

  const finalHeight = spec.autoHeight || spec.presetHeight === 0
    ? contentHeight
    : spec.presetHeight

  canvas.width = spec.width
  canvas.height = finalHeight

  // Background
  ctx.fillStyle = colors.background
  ctx.fillRect(0, 0, spec.width, finalHeight)

  // Image zone (cover-fit, centred crop)
  if (spec.image) {
    const srcRatio = spec.image.naturalWidth / spec.image.naturalHeight
    const dstRatio = spec.width / imageHeight
    let sx = 0, sy = 0, sw = spec.image.naturalWidth, sh = spec.image.naturalHeight
    if (srcRatio > dstRatio) {
      sw = Math.round(sh * dstRatio)
      sx = Math.round((spec.image.naturalWidth - sw) / 2)
    } else {
      sh = Math.round(sw / dstRatio)
      sy = Math.round((spec.image.naturalHeight - sh) / 2)
    }
    ctx.drawImage(spec.image, sx, sy, sw, sh, 0, 0, spec.width, imageHeight)
  }

  // Gradient fade at image/text boundary
  const grad = ctx.createLinearGradient(0, imageHeight - 24, 0, imageHeight + 8)
  grad.addColorStop(0, 'rgba(0,0,0,0)')
  grad.addColorStop(1, colors.background)
  ctx.fillStyle = grad
  ctx.fillRect(0, imageHeight - 24, spec.width, 32)

  // Title
  let y = imageHeight + PAD
  ctx.font = `bold ${spec.titleSize}px "${spec.titleFont}", serif`
  ctx.fillStyle = colors.title
  for (const line of titleLines) {
    ctx.fillText(line, PAD, y)
    y += TITLE_LINE_HEIGHT
  }

  y += Math.round(PAD * 0.5)

  // Summary
  ctx.font = `${spec.bodySize}px "${spec.bodyFont}", sans-serif`
  ctx.fillStyle = colors.body
  for (const line of summaryLines) {
    ctx.fillText(line, PAD, y)
    y += BODY_LINE_HEIGHT
  }

  // Attribution
  if (spec.showAttribution && spec.domain) {
    const attrY = finalHeight - 16
    ctx.fillStyle = colors.separator
    ctx.fillRect(PAD, attrY - 20, spec.width - PAD * 2, 1)
    ctx.font = `13px sans-serif`
    ctx.fillStyle = colors.attribution
    ctx.fillText(spec.domain, PAD, attrY)
  }

  return fits
}
