// src/lib/fonts.ts

export interface FontOption {
  value: string
  label: string
}

const CURATED_GOOGLE_FONTS = [
  'Playfair Display', 'Merriweather', 'Lora', 'Roboto', 'Inter',
  'Open Sans', 'Source Sans Pro', 'Nunito', 'Raleway', 'Montserrat',
]

const GENERIC_FALLBACKS = ['serif', 'sans-serif', 'monospace']

/**
 * Build the font picker option list.
 * Order: detected fonts first (marked "✦"), then curated Google Fonts, then generic fallbacks.
 * Detected fonts that match a curated font are not duplicated.
 *
 * @param headingFamily - The detected heading font family, or null if not detected.
 * @param bodyFamily - The detected body font family, or null if not detected.
 * @return Array of font options in display order.
 */
export function buildFontList(
  headingFamily: string | null,
  bodyFamily: string | null
): FontOption[] {
  const detected = [headingFamily, bodyFamily]
    .filter((f): f is string => f !== null)
    .filter((f, i, arr) => arr.indexOf(f) === i) // deduplicate

  const curated = CURATED_GOOGLE_FONTS.filter(f => !detected.includes(f))
  const generics = GENERIC_FALLBACKS.filter(f => !detected.includes(f))

  return [
    ...detected.map(f => ({ value: f, label: `${f} ✦` })),
    ...curated.map(f => ({ value: f, label: f })),
    ...generics.map(f => ({ value: f, label: f })),
  ]
}

/**
 * Inject a Google Fonts <link> into the document head so Canvas can use
 * the fonts immediately via CSS font-family name.
 * Skips injection if the URL is already present.
 *
 * @param url - The Google Fonts stylesheet URL to inject.
 */
export function injectGoogleFonts(url: string): void {
  if (document.querySelector(`link[href="${url}"]`)) return
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = url
  document.head.appendChild(link)
}
