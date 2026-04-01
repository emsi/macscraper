import { describe, it, expect } from 'vitest'
import { substituteTemplate } from './template'

describe('substituteTemplate', () => {
  it('replaces known variables', () => {
    const result = substituteTemplate('Hello {{title}}!', { title: 'World' })
    expect(result).toBe('Hello World!')
  })

  it('replaces multiple variables', () => {
    const result = substituteTemplate(
      'Title: {{title}}\nText: {{article_text}}',
      { title: 'My Post', article_text: 'Body here' }
    )
    expect(result).toBe('Title: My Post\nText: Body here')
  })

  it('leaves unknown variables as-is', () => {
    const result = substituteTemplate('Hello {{unknown}}!', { title: 'World' })
    expect(result).toBe('Hello {{unknown}}!')
  })

  it('handles empty vars object', () => {
    const result = substituteTemplate('Hello {{title}}!', {})
    expect(result).toBe('Hello {{title}}!')
  })
})
