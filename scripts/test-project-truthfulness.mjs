import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const root = process.cwd()
const read = (relativePath) => readFileSync(join(root, relativePath), 'utf8')
const failures = []

for (const relativePath of ['README.md', 'package.json', 'src-tauri/Cargo.toml', 'src-tauri/tauri.conf.json']) {
  const contents = read(relativePath)
  if (/yourusername|authors\s*=\s*\["you"\]|QEMU 8\.0\+|"csp"\s*:\s*null/i.test(contents)) {
    failures.push(`${relativePath} contains an unreviewed template or fabricated runtime value`)
  }
}

if (!existsSync(join(root, 'package-lock.json')) || !existsSync(join(root, 'src-tauri/Cargo.lock'))) {
  failures.push('both application dependency lock files must be committed')
}

if (/^Cargo\.lock$/m.test(read('.gitignore'))) {
  failures.push('Cargo.lock must not be ignored for this application')
}

const tauriConfig = JSON.parse(read('src-tauri/tauri.conf.json'))
if (typeof tauriConfig.app?.security?.csp !== 'string' || tauriConfig.app.security.csp.length === 0) {
  failures.push('the desktop content security policy must be explicit')
}

function countTests(directory) {
  return readdirSync(directory, { withFileTypes: true }).reduce((count, entry) => {
    const entryPath = join(directory, entry.name)
    if (entry.isDirectory()) return count + countTests(entryPath)
    return count + Number(/\.(test|spec)\.(ts|tsx)$/.test(entry.name) && statSync(entryPath).isFile())
  }, 0)
}

if (countTests(join(root, 'src')) === 0) {
  failures.push('the frontend test suite must contain at least one test file')
}

if (failures.length > 0) {
  throw new Error(failures.join('\n'))
}

console.log('Project truthfulness checks passed.')
