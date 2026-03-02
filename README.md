# dev-container

A minimal, programming-focused dev-container based on **Debian Trixie Slim**.
Zellij is compiled from source in a multi-stage Docker build.

## Included tools

| Tool      | Purpose                              |
|-----------|--------------------------------------|
| zellij    | Terminal multiplexer (compiled from source) |
| vim       | Editor with programming config       |
| git       | Version control                      |
| lazygit   | Git TUI                              |
| curl      | HTTP client                          |
| jq        | JSON processor                       |
| tzdata    | Timezone data                        |
| ca-certs  | CA certificates                      |
| locales   | Locale support (en_US.UTF-8)         |

## File structure

```
.
├── Dockerfile
├── docker-compose.yml
└── config/
    ├── vimrc                          # Vim configuration
    ├── gitconfig                      # Git defaults
    └── zellij/
        ├── config.kdl                 # Zellij main config
        ├── themes/
        │   └── gruvbox.kdl            # Gruvbox dark theme
        └── layouts/
            └── dev.kdl                # Default dev layout
```

## Build & run

```bash
# Build the image (takes a few minutes — Zellij compiles from source)
docker compose build

# Start the container
docker compose up -d
docker compose exec devcontainer zellij --layout dev

# Or run directly without compose
docker build -t devcontainer .
docker run -it --rm -v $(pwd)/workspace:/workspace devcontainer
```

## Zellij key bindings

All bindings use **Alt** as the primary modifier to avoid conflicts with
terminal applications running inside panes.

| Key          | Action                                 |
|--------------|----------------------------------------|
| `Alt s`      | Open **Strider** file picker (float)   |
| `Alt m`      | Open session manager (float)           |
| `Alt n`      | New pane                               |
| `Alt d`      | New pane (split down)                  |
| `Alt r`      | New pane (split right)                 |
| `Alt x`      | Close focused pane                     |
| `Alt f`      | Toggle fullscreen                      |
| `Alt h/j/k/l`| Navigate panes                        |
| `Alt [/]`    | Previous / next tab                    |
| `Alt t`      | New tab                                |
| `Alt w`      | Close tab                              |
| `Alt 1-5`    | Go to tab N                            |
| `Alt u`      | Enter scroll mode                      |
| `Alt /`      | Search scrollback                      |
| `Ctrl q`     | Quit Zellij                            |

### Strider (filepicker)

Press `Alt s` to open Strider as a floating pane. Navigate with arrow keys,
`Enter` opens a file or directory. Press `Escape` or `Ctrl c` to close.

## Default layout tabs

- **dev** — Strider sidebar | editor (vim) top-right | terminal bottom-right
- **git** — Full-pane lazygit
- **shell** — Clean bash terminal

## Vim quick reference

| Key           | Action                        |
|---------------|-------------------------------|
| `<Space>w`    | Save                          |
| `<Space>q`    | Quit                          |
| `<Space>e`    | Open netrw file explorer      |
| `<Space>n/p`  | Next / previous buffer        |
| `Ctrl-h/j/k/l`| Navigate splits              |
| `<Space>/`    | Clear search highlight        |

## Customisation

- **Timezone**: change `TZ=Europe/Berlin` in `Dockerfile` and `docker-compose.yml`
- **Default user**: add a `RUN useradd ...` block and switch to that user
- **Additional tools**: add packages to the `apt-get install` line in Stage 2
- **Language runtimes**: extend the `runtime` stage with your language-specific
  installer (pyenv, nvm, sdkman, etc.)
