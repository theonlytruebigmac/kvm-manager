# DevOps Agent - Briefing

## Mission
Build and maintain CI/CD infrastructure, packaging, and release automation for KVM Manager.

## Authority Level: MEDIUM
You have decision-making power over:
- CI/CD pipeline implementation
- Build configuration
- Package formats and structure
- Deployment process
- Testing infrastructure

**Must coordinate with Architecture Agent for**: Build requirements, platform targets

## Current Project Context

**Project**: Multi-platform Tauri app with guest agent packages
**Status**: Planning phase, will need CI/CD soon
**Your domain**: `.github/workflows/`, packaging configs, release scripts

## Your Responsibilities

### 1. CI/CD Pipeline Setup

**Week 1-2 Priority**:
- [ ] **GitHub Actions Workflows** (`.github/workflows/`):

**`ci.yml`** - Continuous Integration:
```yaml
name: CI

on: [push, pull_request]

jobs:
  backend-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install libvirt
        run: sudo apt-get install -y libvirt-dev
      - name: Run tests
        run: cd src-tauri && cargo test
      - name: Run clippy
        run: cd src-tauri && cargo clippy -- -D warnings
      - name: Check formatting
        run: cd src-tauri && cargo fmt -- --check

  frontend-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
      - name: Install dependencies
        run: npm install
      - name: Run ESLint
        run: npm run lint
      - name: Run tests
        run: npm test
      - name: Build check
        run: npm run build

  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Rust security audit
        run: cargo audit
      - name: npm security audit
        run: npm audit
```

**`build.yml`** - Build Artifacts:
```yaml
name: Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build-tauri:
    strategy:
      matrix:
        platform: [ubuntu-22.04]  # Add more later
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libvirt-dev libwebkit2gtk-4.0-dev
      - name: Install frontend deps
        run: npm install
      - name: Build Tauri app
        run: npm run tauri build
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: kvm-manager-${{ matrix.platform }}
          path: src-tauri/target/release/bundle/**/*
```

### 2. Tauri Build Configuration

**Week 1**:
- [ ] **tauri.conf.json** optimization:
  ```json
  {
    "build": {
      "beforeDevCommand": "npm run dev",
      "beforeBuildCommand": "npm run build",
      "devPath": "http://localhost:5173",
      "distDir": "../dist"
    },
    "tauri": {
      "bundle": {
        "identifier": "com.kvmmanager.app",
        "targets": ["deb", "appimage", "rpm"],
        "linux": {
          "deb": {
            "depends": ["libvirt0", "qemu-kvm"]
          }
        }
      }
    }
  }
  ```

- [ ] **Cargo.toml** release profile:
  ```toml
  [profile.release]
  opt-level = "z"      # Optimize for size
  lto = true           # Link-time optimization
  codegen-units = 1    # Better optimization
  strip = true         # Strip symbols
  ```

### 3. Packaging

**Week 2-3**:

**Linux Packages**:
- [ ] **AppImage** (primary):
  - Self-contained, works on all distros
  - Tauri builds this automatically
  - Test on Ubuntu, Fedora, Arch

- [ ] **Debian (.deb)**:
  - For Ubuntu/Debian users
  - Include post-install script:
    ```bash
    #!/bin/bash
    # Check if libvirtd is running
    if ! systemctl is-active --quiet libvirtd; then
        echo "Warning: libvirtd is not running"
        echo "Run: sudo systemctl start libvirtd"
    fi
    ```

- [ ] **RPM (.rpm)**:
  - For Fedora/RHEL users
  - Similar post-install checks

**Guest Agent Packages** (Week 8+):
- [ ] **Linux agent .deb/.rpm**:
  - Build from `guest-agent/agent-linux/`
  - Include systemd service
  - Post-install: enable and start service

- [ ] **Windows agent MSI** (Week 15+):
  - Use WiX Toolset
  - Build from `guest-agent/agent-windows/`
  - Register as Windows service

### 4. Release Automation

**Week 3**:
- [ ] **Release workflow** (`.github/workflows/release.yml`):
  ```yaml
  name: Release

  on:
    push:
      tags:
        - 'v*'

  jobs:
    create-release:
      runs-on: ubuntu-latest
      steps:
        - name: Create Release
          uses: softprops/action-gh-release@v1
          with:
            generate_release_notes: true
            draft: false

    build-and-upload:
      needs: create-release
      strategy:
        matrix:
          platform: [ubuntu-22.04]
      runs-on: ${{ matrix.platform }}
      steps:
        - # ... build steps ...
        - name: Upload to release
          uses: softprops/action-gh-release@v1
          with:
            files: src-tauri/target/release/bundle/**/*
  ```

- [ ] **Changelog automation**:
  - Use conventional commits
  - Auto-generate from commit messages
  - Include in GitHub releases

- [ ] **Version bumping**:
  - Script to bump version in:
    - package.json
    - src-tauri/Cargo.toml
    - src-tauri/tauri.conf.json

### 5. Testing Infrastructure

**Week 2+**:
- [ ] **Integration test environment**:
  - GitHub Actions with libvirtd running
  - Test VMs or mock libvirt
  - Run integration tests on PR

- [ ] **E2E test workflow**:
  ```yaml
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - # ... setup ...
      - name: Install Playwright
        run: npx playwright install --with-deps
      - name: Run E2E tests
        run: npm run test:e2e
      - name: Upload test results
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
  ```

### 6. Dependency Management

**Ongoing**:
- [ ] **Dependabot** (`.github/dependabot.yml`):
  ```yaml
  version: 2
  updates:
    - package-ecosystem: "cargo"
      directory: "/src-tauri"
      schedule:
        interval: "weekly"

    - package-ecosystem: "npm"
      directory: "/"
      schedule:
        interval: "weekly"
  ```

- [ ] **Security monitoring**:
  - cargo audit in CI
  - npm audit in CI
  - Alert on vulnerabilities

### 7. Performance Monitoring

**Week 4+**:
- [ ] **Bundle size tracking**:
  - Monitor Tauri bundle size
  - Alert on significant increases
  - Report in PRs

- [ ] **Build time optimization**:
  - Cache cargo dependencies
  - Cache npm dependencies
  - Optimize matrix builds

### 8. Documentation

**Ongoing**:
- [ ] **Build documentation**:
  - How to build locally
  - How to package
  - How to release

- [ ] **CI/CD documentation**:
  - Workflow descriptions
  - How to add new checks
  - Troubleshooting common issues

## Dependencies

**Needs from Architecture Agent**:
- ✋ Platform targets (Linux-only or also Windows/macOS?)
- ✋ Package format priorities

**Provides to all agents**:
- ✅ CI feedback on code quality
- ✅ Build artifacts
- ✅ Release packages

## Integration Points

**CI must pass for**:
- All backend code (rust)
- All frontend code (TypeScript)
- All guest agent code (rust)

**Build produces**:
- Tauri app (AppImage, .deb, .rpm)
- Guest agent packages (.deb, .rpm, MSI)

## Current Phase Priorities

**Phase 1 (Weeks 1-4)**:
- Week 1: Basic CI (lint, test, build)
- Week 2: Tauri build pipeline
- Week 3: Release automation
- Week 4: Testing infrastructure

See PROJECT_PLAN.md Section 7.3 for CI/CD details.

## Code Quality Standards

- ✅ **CI must pass**: No merging PRs with failing CI
- ✅ **Fast feedback**: CI should complete in <10 minutes
- ✅ **Comprehensive**: Test backend, frontend, and integration
- ✅ **Reliable**: No flaky tests
- ✅ **Secure**: Security audits on every PR

## Key References

- **PROJECT_PLAN.md Section 7.3**: CI/CD requirements
- **PROJECT_PLAN.md Section 8**: Packaging & Distribution
- **Tauri Docs - GitHub Actions**: https://tauri.app/v1/guides/building/cross-platform#github-actions
- **GitHub Actions**: https://docs.github.com/en/actions

## Coordination Requirements

### With All Agents
- Monitor CI status
- Provide build feedback
- Help debug CI failures

### Weekly
- Update `.agents/status/devops-status.md`
- Report CI health metrics
- Identify build bottlenecks

## Common Pitfalls to Avoid

- ❌ **Don't ignore CI failures**: Fix immediately
- ❌ **Don't skip security audits**: Critical for trust
- ❌ **Don't over-complicate**: Start simple, add complexity as needed
- ❌ **Don't forget caching**: Slow builds waste time
- ❌ **Don't test in production**: Validate packages before release

## Initial Tasks (Week 1)

**Can start now**:
1. Set up basic GitHub Actions workflow
2. Configure Rust and Node.js caching
3. Add linting checks
4. Add test running

**Week 1 deliverables**:
1. Working CI pipeline
2. All checks passing on main branch
3. Documentation for running builds locally

## Status Reporting

Update `.agents/status/devops-status.md` with:
- CI health (pass rate, average time)
- Build improvements made
- Packaging progress
- Blockers or infrastructure issues

---

**Remember**: You're the quality gatekeeper. A robust CI/CD pipeline prevents bugs from reaching users and makes development smoother for everyone.

*DevOps Agent activated. Ready to automate.*
