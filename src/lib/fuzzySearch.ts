/**
 * Simple fuzzy search implementation
 * Matches characters in order but allows gaps between them
 */

interface FuzzyMatch {
  score: number
  matches: number[]  // Indices of matched characters
}

/**
 * Check if pattern fuzzy matches the text
 * Returns null if no match, or a match object with score and positions
 */
export function fuzzyMatch(pattern: string, text: string): FuzzyMatch | null {
  if (!pattern) return { score: 1, matches: [] }
  if (!text) return null

  pattern = pattern.toLowerCase()
  text = text.toLowerCase()

  let patternIdx = 0
  let textIdx = 0
  const matches: number[] = []
  let score = 0
  let consecutiveBonus = 0

  while (patternIdx < pattern.length && textIdx < text.length) {
    if (pattern[patternIdx] === text[textIdx]) {
      matches.push(textIdx)

      // Scoring:
      // - Base point for each match
      score += 1

      // - Bonus for consecutive matches
      if (matches.length > 1 && matches[matches.length - 1] === matches[matches.length - 2] + 1) {
        consecutiveBonus += 2
      }

      // - Bonus for matching at word boundaries
      if (textIdx === 0 || text[textIdx - 1] === ' ' || text[textIdx - 1] === '-' || text[textIdx - 1] === '_') {
        score += 3
      }

      patternIdx++
    }
    textIdx++
  }

  // All pattern characters must be found
  if (patternIdx !== pattern.length) {
    return null
  }

  // Add consecutive bonus
  score += consecutiveBonus

  // Normalize score by text length (prefer shorter matches)
  score = score / (text.length * 0.1)

  return { score, matches }
}

/**
 * Search through an array of items with fuzzy matching
 */
export function fuzzySearch<T>(
  items: T[],
  pattern: string,
  getSearchableText: (item: T) => string[]
): { item: T; score: number; matches: Map<string, number[]> }[] {
  if (!pattern.trim()) {
    return items.map(item => ({ item, score: 1, matches: new Map() }))
  }

  const results: { item: T; score: number; matches: Map<string, number[]> }[] = []

  for (const item of items) {
    const searchableTexts = getSearchableText(item)
    let bestScore = 0
    const allMatches = new Map<string, number[]>()

    for (const text of searchableTexts) {
      const match = fuzzyMatch(pattern, text)
      if (match) {
        if (match.score > bestScore) {
          bestScore = match.score
        }
        allMatches.set(text, match.matches)
      }
    }

    if (bestScore > 0) {
      results.push({ item, score: bestScore, matches: allMatches })
    }
  }

  // Sort by score descending
  results.sort((a, b) => b.score - a.score)

  return results
}

/**
 * Simple substring search (fallback for simple queries)
 */
export function substringSearch<T>(
  items: T[],
  pattern: string,
  getSearchableText: (item: T) => string[]
): T[] {
  if (!pattern.trim()) return items

  const lowerPattern = pattern.toLowerCase()

  return items.filter(item => {
    const texts = getSearchableText(item)
    return texts.some(text => text.toLowerCase().includes(lowerPattern))
  })
}
