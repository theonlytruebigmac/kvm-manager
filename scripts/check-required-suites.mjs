import { readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const root = process.cwd()

function countFiles(directory, matcher) {
  return readdirSync(directory, { withFileTypes: true }).reduce((count, entry) => {
    const entryPath = join(directory, entry.name)
    if (entry.isDirectory()) return count + countFiles(entryPath, matcher)
    return count + Number(matcher(entry.name) && statSync(entryPath).isFile())
  }, 0)
}

const expectedSuites = [
  {
    name: 'frontend test files',
    directory: join(root, 'src'),
    matcher: (name) => /\.(test|spec)\.(ts|tsx)$/.test(name),
    minimum: 4,
  },
  {
    name: 'Rust integration test files',
    directory: join(root, 'src-tauri', 'tests'),
    matcher: (name) => name.endsWith('.rs'),
    minimum: 7,
  },
]

const failures = expectedSuites.flatMap(({ name, directory, matcher, minimum }) => {
  const found = countFiles(directory, matcher)
  return found >= minimum ? [] : [`Expected at least ${minimum} ${name}; found ${found}.`]
})

if (failures.length > 0) throw new Error(failures.join('\n'))

console.log('Required test-suite counts are present.')
