import * as esbuild from 'esbuild'

// Bundle the ES module source files from public/novnc
await esbuild.build({
  entryPoints: ['public/novnc/rfb.js'],
  bundle: true,
  format: 'esm',
  outfile: 'public/novnc-bundle.js',
  platform: 'browser',
  target: 'esnext',
  supported: {
    'top-level-await': true,
  },
  banner: {
    js: '// noVNC bundle - auto-generated'
  }
})

console.log('noVNC bundle created at public/novnc-bundle.js')
