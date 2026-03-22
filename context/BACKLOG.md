# Backlog

Prioritized work items. GitHub issues referenced where they exist.
Source of truth — GitHub issues are for external visibility.

## Completed

- [x] **dev-box sync** — Theme switching without manual file deletion (#25)
- [x] **Shell enhancement tools** — ripgrep, fd, bat, eza, zoxide, fzf, delta + aliases
- [x] **Starship prompt** — Installed in base image (#28)
- [x] **Keyboard shortcuts cheatsheet** — Docs page (#16)
- [x] **generate deprecated** — Replaced by sync (#25)
- [x] **AI provider flexibility** — Claude/Aider/Gemini, dynamic layouts, optional (#19)
- [x] **Process templates** — release, code-review, feature-dev, bug-fix (#29)
- [x] **SKILL.md support** — 3 example skills scaffolded (#30)
- [x] **Addon packages** — infrastructure, kubernetes, cloud-aws/gcp/azure, 6 docs addons (#18)
- [x] **Security audit command** — dev-box audit: cargo audit, pip-audit, trivy (#24)
- [x] **Zensical migration** — Config + maintain.sh support (#26)
- [x] **Dockerfile optimization** — Pinned versions, fontconfig fix, COPY consolidation (#27)
- [x] **Version in docs header** — site_name includes version (#12)

## Next Up

- [ ] **Theming screenshots** — Gallery for all 6 themes in docs (#14)
- [ ] **Starship prompt presets** — Configurable presets in dev-box.toml (#17)
- [ ] **Additional image flavors** — node, go, python-rust
- [ ] **Curated skill library** — Expand to 50-100 vetted skills (#30)

## Planned — Architecture

- [ ] **Plugin / extension system** — Hooks, template overrides, community features (#20)
- [ ] **Zellij plugin integration** — zjstatus, custom dev-box status plugin (#21)
- [ ] **Automated context migration** — Safe auto-migration + AI-assisted prompts
- [ ] **Multi-service support** — Additional docker-compose services, dev-box ps/logs
- [ ] **Remote development** — Environments on remote hosts

## Quality & Security

- [ ] **Code review for simplification** — Dedup, test helpers, dead code (#22)
- [ ] **Security review** — Input validation, container security, supply chain (#23)
- [ ] **TeX Live builder deduplication** — 3 Dockerfiles share identical 90-line stage
- [ ] **Binary checksum verification** — Downloaded binaries in base Dockerfile
- [ ] **Image signing** — sigstore/cosign for published images

## Ideas

- [ ] LaTeX addon package groups (music, chemistry, linguistics)
- [ ] Agent orchestration tools — too volatile, revisit later
- [ ] AIUC-1 compliance alignment (awareness for dev-box)
