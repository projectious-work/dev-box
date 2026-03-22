# Backlog

Prioritized work items. GitHub issues referenced where they exist.
Source of truth — GitHub issues are for external visibility.

## Next Up (v0.5.0)

- [ ] **dev-box sync** — Reconcile config changes (re-seed, regenerate). Theme switching without manual file deletion (#25)
- [ ] **Shell enhancement tools** — ripgrep, fd, bat, eza, zoxide, fzf, delta in base image + shell aliases
- [ ] **Starship prompt** — Install in base image, themed presets, 10 preset choices (#28)
- [ ] **Zensical migration** — Migrate docs from MkDocs before EOL Nov 2026 (#26)
- [ ] **Keyboard shortcuts cheatsheet** — Docs page for Zellij, Yazi, Vim, lazygit (#16)
- [ ] **Dockerfile optimization** — Cache mounts, layer reduction, version pinning (#27)
- [ ] **Theming screenshots** — Gallery for all 6 themes in docs (#14)

## Planned — v0.6.0 (Flexibility)

- [ ] **AI provider flexibility** — Optional/selectable: Claude, Aider, Gemini, Codex, Goose. Stacked panes. No pane if none (#19)
- [ ] **Addon packages** — infrastructure (OpenTofu, Ansible, Packer), kubernetes (kubectl, Helm, k9s), cloud-aws/gcp/azure (#18)
- [ ] **Process templates** — Standard process docs (release, code-review, feature-dev, bug-fix) in context/processes/ (#29)
- [ ] **SKILL.md support** — Skill directory, vetted skill library (50-100 curated skills), skill install command (#30)
- [ ] **Additional image flavors** — node, go, python-rust, python-node

## Planned — v0.7.0+ (Architecture)

- [ ] **Plugin / extension system** — Hooks, template overrides, community features (#20)
- [ ] **Zellij plugin integration** — zjstatus, custom dev-box status plugin (#21)
- [ ] **Automated context migration** — Safe auto-migration + AI-assisted prompts for breaking changes
- [ ] **Multi-service support** — Additional docker-compose services, dev-box ps/logs
- [ ] **Remote development** — Run environments on remote hosts

## v1.0.0 Requirements (see context/project-notes/v1-readiness.md)

- [ ] **Versioning policy** — Document what's breaking vs non-breaking
- [ ] **API stability** — Commit to backward compat for dev-box.toml schema and CLI
- [ ] **Preview mode** — `--preview` flag for experimental features
- [ ] **Security scanning** — cargo audit, Trivy, SBOM generation (#24)
- [ ] **Security review** — Input validation, container security, supply chain (#23)
- [ ] **Code review** — Simplification, dedup, test coverage (#22)
- [ ] **Complete documentation** — All features, migration guide, architecture overview
- [ ] **Image signing** — sigstore/cosign for published images
- [ ] **Real-world validation** — Multiple derived projects battle-tested

## Documentation

- [x] **Version display in docs header** — Done (#12)
- [ ] **VS Code settings conflict detection** — Migration instructions check .vscode/settings.json
- [ ] **Bash prompt themes docs** — Screenshots of Starship presets (#17)

## Ideas / Investigation

- [ ] LaTeX addon package groups (music, chemistry, linguistics) vs documenting additions
- [ ] Agent orchestration tools — too volatile, revisit in 6-12 months
- [ ] Python package manager choice — uv is correct default, keep as-is
- [ ] AIUC-1 compliance alignment (relevant for kaits, awareness for dev-box)
- [ ] ClawHub / external skill marketplaces — user responsibility, not dev-box managed
