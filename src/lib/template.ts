// src/lib/template.ts

/**
 * Replace {{varName}} placeholders in a template string.
 * Unknown variables are left as-is (no silent failure).
 *
 * :param template: The template string with {{varName}} placeholders.
 * :param vars: A record of variable names to replacement values.
 * :return: The template with all known variables replaced.
 */
export function substituteTemplate(
  template: string,
  vars: Record<string, string>
): string {
  return template.replace(/\{\{(\w+)\}\}/g, (match, key) => vars[key] ?? match)
}
