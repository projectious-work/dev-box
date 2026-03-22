# Backlog

Prioritized work items. GitHub issues referenced where they exist.

## In Progress

- [ ] **Theming screenshots** — Screenshot gallery for all 6 themes across 4 tools in docs (#14)

## Next Up

- [ ] **dev-box sync** — Reconcile config changes (re-seed affected files, regenerate). Enables theme switching without manual file deletion (#25)
- [ ] **Zensical migration** — Migrate docs from MkDocs to Zensical before MkDocs 1.x EOL Nov 2026 (#26)
- [ ] **Shell enhancement tools** — Install ripgrep, fd, bat, eza, zoxide, fzf, delta in base image
- [ ] **Starship prompt** — Install Starship in base image, seed starship.toml with themed presets, 10 preset choices (#28)

## Planned — Features

- [ ] **AI provider flexibility** — Make AI optional/selectable: Claude, Aider, Gemini CLI, Codex. Multiple stacked in one pane. No pane if none selected (#19)
- [ ] **Addon packages** — Selectable tool bundles for generated Dockerfile: infrastructure (OpenTofu, Ansible, Packer), kubernetes (kubectl, Helm, k9s, Kustomize), cloud-aws, cloud-gcp, cloud-azure (#18)
- [ ] **Keyboard shortcuts cheatsheet** — Comprehensive docs page for Zellij, Yazi, Vim, lazygit bindings (#16)
- [ ] **Bash prompt themes** — Starship preset choices in dev-box.toml, documented with screenshots (#17)
- [ ] **Additional image flavors** — node, go, python-rust, python-node
- [ ] **Automated context migration** — Automate safe migrations, AI-assisted prompts for breaking changes

## Planned — Architecture

- [ ] **Plugin / extension system** — Hooks, template overrides, community features (#20)
- [ ] **Zellij plugin integration** — Evaluate zjstatus, yazelix; consider custom dev-box status plugin (#21)
- [ ] **Multi-service support** — Additional services in docker-compose, dev-box ps/logs
- [ ] **Remote development** — Run environments on remote hosts

## Quality & Security

- [ ] **Code review for simplification** — Deduplication, test helper extraction, dead code removal (#22)
- [ ] **Security review** — Input validation, container security, supply chain audit (#23)
- [ ] **Security scanning** — CVE checking (Trivy, cargo audit), SBOM generation, binary checksum verification (#24)
- [ ] **Dockerfile best practices** — Layer optimization, cache mounts, pinned versions, size reduction (#27)

## Documentation

- [ ] **VS Code settings conflict detection** — Migration instructions should check .vscode/settings.json vs devcontainer.json
- [ ] **Version display in docs header** — Done (v0.4.2 in site_name) (#12)

## Ideas / Investigation

- [ ] LaTeX addon package groups (music, chemistry, linguistics) vs documenting common additions
- [ ] Agent orchestration tools — too volatile, revisit in 6-12 months
- [ ] Python package manager choice — uv is correct default, no need to offer alternatives
