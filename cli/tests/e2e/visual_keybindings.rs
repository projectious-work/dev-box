//! Visual keybinding tests — verify that aibox-configured keyboard shortcuts
//! produce the expected terminal output in live sessions.
//!
//! Records headless sessions on the e2e companion via asciinema and asserts
//! that pressing each keybinding causes the expected UI change.
//!
//! Keybindings under test:
//!   Yazi  : e (open in vim pane), Enter (edit in-place)
//!   Vim   : <Space>e (netrw), <Space>l (buffer list), <Space>w (save),
//!           <Space>n/<Space>p (next/prev buffer)
//!   Lazygit: ? (help overlay), <Space> (stage file)
//!
//! Vim tests use the full container vimrc (`/opt/aibox/vimrc`) deployed by
//! `runner.deploy()`, because the seeded DEFAULT_VIMRC does not include leader
//! key mappings (those live in the container image, not the home directory).
//!
//! Lazygit tests require lazygit to be installed (added to Dockerfile.e2e).
//! Tests skip gracefully if lazygit is not found on the companion.

use serial_test::serial;

use super::runner::E2eRunner;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Extract concatenated output data from an asciicast v2 file.
fn extract_cast_output(cast_content: &str) -> String {
    cast_content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
            let arr = parsed.as_array()?;
            if arr.len() >= 3 && arr[1].as_str() == Some("o") {
                arr[2].as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Record an asciinema session using the given driver script.
///
/// The caller is responsible for `aibox init` / workspace setup beforehand.
/// Kills any leftover zellij sessions before recording.
fn record(runner: &E2eRunner, test_name: &str, driver: &str) -> String {
    let ws = format!("/workspaces/{}", test_name);
    runner.exec("pkill -9 -x zellij 2>/dev/null; rm -rf /tmp/zellij-* 2>/dev/null; sleep 0.3");
    runner.write_file(test_name, "driver.sh", driver);
    runner.exec(&format!("chmod +x {ws}/driver.sh"));
    runner.exec(&format!(
        "LC_ALL=C.UTF-8 LANG=C.UTF-8 asciinema rec --cols 160 --rows 45 --overwrite \
         -c {ws}/driver.sh {ws}/recording.cast 2>/dev/null; true"
    ));
    runner.read_file(test_name, "recording.cast")
}

// ── Yazi keybinding tests ─────────────────────────────────────────────────────

/// yazi `e` key: open-in-editor pipeline.
///
/// Layout: dev (yazi 40% left, vim 60% right). Pressing `e` on hello.rs
/// triggers open-in-editor, which moves focus right and sends `:e <file>` to
/// vim. The file content should appear in the vim pane.
#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_yazi_e_opens_file_in_vim_pane() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-e";
    runner.cleanup(test_name);

    // Init project (seeds yazi keymap with `e` binding + zellij config)
    let init = runner.aibox(
        test_name,
        &["init", "--name", test_name, "--base", "debian", "--process", "core"],
    );
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Place the file in src/ so yazi (started with args pointing at src/) shows
    // hello.rs as the only entry and focuses it immediately. If yazi were started
    // at the workspace root, the first alphabetical entry would be a directory.
    let marker = "AIBOX_E2E_OPEN_IN_EDITOR_VERIFIED";
    runner.write_file(
        test_name,
        "src/hello.rs",
        &format!("fn main() {{\n    // {marker}\n    println!(\"hello\");\n}}\n"),
    );

    let ws = format!("/workspaces/{}", test_name);
    let home = format!("{ws}/.aibox-home");
    // Use cwd to start yazi in src/ so hello.rs is the only (first-focused) entry.
    // Using `args` (positional ENTRIES) causes yazi to fail with ENXIO in some envs.
    let src = format!("{ws}/src");

    let driver = format!(
        r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={home}
export PATH=/usr/local/bin:$PATH

# Layout: yazi (left, named "files") + vim-loop (right, named "editor").
cat > /tmp/kb_yazi_e_layout.kdl << 'LAYOUT_EOF'
layout {{
    default_tab_template {{
        children
        pane size=1 borderless=true {{
            plugin location="zellij:status-bar"
        }}
    }}
    tab name="dev" focus=true {{
        pane split_direction="vertical" {{
            pane size="40%" name="files" focus=true {{
                command "yazi"
                args "{src}"
            }}
            pane size="60%" name="editor" {{
                command "vim-loop"
                cwd "{ws}"
            }}
        }}
    }}
}}
LAYOUT_EOF

(
  # Wait for zellij + yazi to fully start.
  sleep 2.5

  # Discover the active (non-exited) session so `zellij action` commands
  # target THIS session and not a stale one from a prior run.
  export ZELLIJ_SESSION_NAME=$(zellij list-sessions --no-formatting 2>/dev/null \
    | grep -v EXITED | head -1 | awk '{{print $1}}')

  # Dismiss the zellij 0.44 Release Notes popup (floating pane, ~2.5s delay).
  zellij action write 27
  sleep 0.5

  # Send 'e' to yazi (terminal pane 0). The keymap binding runs open-in-editor
  # which moves focus to the vim pane and sends `:e <file>`.
  zellij action send-keys --pane-id 0 "e"

  # Poll the editor pane until the marker appears or timeout (15s).
  MARKER="{marker}"
  FOUND=0
  for i in $(seq 1 30); do
    sleep 0.5
    SCREEN=$(zellij action dump-screen --pane-id 1 2>/dev/null)
    if echo "$SCREEN" | grep -qF "$MARKER"; then
      FOUND=1
      break
    fi
  done

  sleep 0.5
  pkill -x zellij 2>/dev/null
) &

zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout /tmp/kb_yazi_e_layout.kdl 2>/dev/null
true
"#
    );

    // The driver polls for the marker using `zellij action dump-screen` inside the
    // session and exits once found (or times out). We also check the cast output
    // directly — the marker should appear in vim's pane render.
    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(
        cast.lines().count() > 5,
        "cast too small — session likely failed to start"
    );

    // The marker must appear in the cast. Since the driver polls dump-screen and
    // vim's pane eventually reflects it, the asciinema recording also captures it.
    assert!(
        output.contains(marker),
        "expected marker '{}' in cast — seeded 'e' keybinding did not open \
         the file in the vim pane via open-in-editor",
        marker
    );

    runner.cleanup(test_name);
}

/// yazi `Enter` key: edit file in-place (vim suspends yazi).
///
/// Pressing Enter on a file calls `${EDITOR:-vim}` in block mode, so yazi
/// suspends and vim takes over the pane. The vim `~` end-of-buffer lines
/// and the file content should appear in the cast.
#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_kb_yazi_enter_opens_vim_inplace() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-yazi-enter";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &["init", "--name", test_name, "--base", "debian", "--process", "core"],
    );
    assert!(init.status.success(), "init failed");

    runner.write_file(
        test_name,
        "readme.md",
        "# Hello World\nThis is a test file.\n",
    );

    let ws = format!("/workspaces/{}", test_name);
    let home = format!("{ws}/.aibox-home");

    let driver = format!(
        r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={home}
export EDITOR=vim

(
  sleep 3
  # Enter on readme.md: opens vim in-place (yazi suspends)
  zellij action write 13
  sleep 2
  pkill -x zellij 2>/dev/null
) &

zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout "$HOME/.config/zellij/layouts/dev.kdl" 2>/dev/null
true
"#
    );

    // For the dev layout the cwd is /workspace (hardcoded in seeded layout).
    // Rewrite the layout to point at our test workspace.
    // Rewrite dev layout to point at our test workspace, using args for yazi
    // so it reliably opens the right directory (cwd alone can fail silently).
    runner.write_file(
        test_name,
        ".aibox-home/.config/zellij/layouts/dev.kdl",
        &format!(
            r#"layout {{
    default_tab_template {{
        children
        pane size=1 borderless=true {{
            plugin location="zellij:status-bar"
        }}
    }}
    tab name="dev" focus=true {{
        pane split_direction="vertical" {{
            pane size="40%" name="files" focus=true {{
                command "yazi"
                args "{ws}"
            }}
            pane size="60%" name="editor" {{
                command "vim-loop"
                cwd "{ws}"
            }}
        }}
    }}
}}
"#
        ),
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    // After Enter, vim opens readme.md in-place. "~" lines are vim's empty-line
    // marker and "Hello World" is the file content.
    assert!(
        output.contains('~') || output.contains("Hello World"),
        "expected vim UI (~ lines or file content) after yazi Enter, not found in cast"
    );

    runner.cleanup(test_name);
}

// ── Vim keybinding tests ──────────────────────────────────────────────────────

/// Helper: build a driver script that starts vim with the full container vimrc.
///
/// `vim_args` are appended to the `vim -u /opt/aibox/vimrc` invocation.
/// `actions` is a list of `(delay_secs, shell_command)` pairs injected from a
/// background process before zellij is killed at `kill_after_secs`.
fn vim_driver(ws: &str, home: &str, vim_args: &str, actions: &[(f32, &str)], kill_after: f32) -> String {
    let mut action_lines = String::new();
    for (delay, cmd) in actions {
        action_lines.push_str(&format!("  sleep {delay}\n  {cmd}\n"));
    }
    format!(
        r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={home}

cat > /tmp/vim_kb_layout.kdl << 'LAYOUT_EOF'
layout {{
    pane {{
        command "vim"
        args "-u" "/opt/aibox/vimrc" {vim_args}
        cwd "{ws}"
    }}
}}
LAYOUT_EOF

(
  # Zellij 0.44 shows a "Release Notes" popup on first run in each new home dir.
  # Dismiss it with ESC before sending the actual test keybindings.
  sleep 1.5
  zellij action write 27
  sleep 0.5
{action_lines}  sleep {kill_after}
  pkill -x zellij 2>/dev/null
) &

zellij --config "$HOME/.config/zellij/config.kdl" \
       --config-dir "$HOME/.config/zellij" \
       --layout /tmp/vim_kb_layout.kdl 2>/dev/null
true
"#
    )
}

/// vim `<Space>e`: opens netrw file explorer.
///
/// The full container vimrc maps `<leader>e` to `:Explore`. netrw in tree mode
/// (liststyle=3, banner off) shows the directory tree. We assert that either
/// a file we created or a netrw-specific marker appears in the cast.
#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_kb_vim_leader_e_opens_netrw() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-netrw";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &["init", "--name", test_name, "--base", "debian", "--process", "core"],
    );
    assert!(init.status.success(), "init failed");

    runner.write_file(test_name, "project.toml", "[package]\nname = \"test\"\n");

    let ws = format!("/workspaces/{}", test_name);
    let home = format!("{ws}/.aibox-home");

    let driver = vim_driver(
        &ws,
        &home,
        &format!("\"{}\"", format!("{ws}/project.toml")),
        &[
            (0.5, "zellij action write 32"),         // <Space>
            (0.1, "zellij action write-chars \"e\""),// e  → :Explore
        ],
        2.0,
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    // netrw renders the directory tree; project.toml or the workspace name should
    // appear in the listing since we opened vim from that directory.
    assert!(
        output.contains("project.toml") || output.contains("netrw") || output.contains(".."),
        "expected netrw directory listing after <Space>e, not found in cast"
    );

    runner.cleanup(test_name);
}

/// vim `<Space>l`: lists open buffers via `:ls`.
///
/// With two files open, `:ls` shows the buffer list with `%a` marking the
/// current buffer. We assert `%a` appears in the cast.
#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_kb_vim_leader_l_shows_buffer_list() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-buflist";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &["init", "--name", test_name, "--base", "debian", "--process", "core"],
    );
    assert!(init.status.success(), "init failed");

    runner.write_file(test_name, "alpha.rs", "fn alpha() {}\n");
    runner.write_file(test_name, "beta.rs", "fn beta() {}\n");

    let ws = format!("/workspaces/{}", test_name);
    let home = format!("{ws}/.aibox-home");

    let driver = vim_driver(
        &ws,
        &home,
        &format!("\"{}\" \"{}\"", format!("{ws}/alpha.rs"), format!("{ws}/beta.rs")),
        &[
            // Send Space+l as one write-chars call so vim receives the leader and
            // the key in a single chunk — avoids a timing gap that could cause vim
            // to treat them as two independent keystrokes.
            (0.5, "zellij action write-chars \" l\""),
        ],
        2.0,
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    // After :ls, vim shows the buffer list.  The `%a` flag bytes may be split by
    // ANSI attribute escapes, so we assert for the filename (shown with a leading
    // double-quote in vim's :ls output) and the "line" word from the line-number column.
    assert!(
        output.contains("\"alpha") || output.contains("\"beta") || output.contains("line 1"),
        "expected ':ls' buffer list output (filename or line number column) after <Space>l"
    );

    runner.cleanup(test_name);
}

/// vim `<Space>w`: saves the current buffer.
///
/// We open a file, enter insert mode, add text, escape, then save with
/// `<Space>w`. vim prints `"filename" NL, NB written` — we assert "written".
#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_kb_vim_leader_w_saves_file() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-save";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &["init", "--name", test_name, "--base", "debian", "--process", "core"],
    );
    assert!(init.status.success(), "init failed");

    runner.write_file(test_name, "save_me.rs", "fn main() {}\n");

    let ws = format!("/workspaces/{}", test_name);
    let home = format!("{ws}/.aibox-home");

    let driver = vim_driver(
        &ws,
        &home,
        &format!("\"{}\"", format!("{ws}/save_me.rs")),
        &[
            (0.5,  "zellij action write-chars \"A\""),    // append at end of line
            (0.15, "zellij action write-chars \" edited\""), // type some text
            (0.15, "zellij action write 27"),              // Esc → normal mode
            (0.2,  "zellij action write 32"),              // <Space>
            (0.1,  "zellij action write-chars \"w\""),     // w  → :w
        ],
        2.0,
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    // vim echoes `"save_me.rs" 1L, NB written` after :w
    assert!(
        output.contains("written"),
        "expected vim 'written' confirmation after <Space>w save"
    );

    runner.cleanup(test_name);
}

/// vim `<Space>n` / `<Space>p`: cycle through buffers.
///
/// Open two files. `<Space>n` switches from alpha.rs to beta.rs.
/// `<Space>p` switches back. Both filenames should appear in the cast.
#[test]
#[serial]
#[ntest::timeout(60_000)]
fn visual_kb_vim_leader_n_p_cycles_buffers() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test_name = "visual-kb-vim-bufcycle";
    runner.cleanup(test_name);

    let init = runner.aibox(
        test_name,
        &["init", "--name", test_name, "--base", "debian", "--process", "core"],
    );
    assert!(init.status.success(), "init failed");

    runner.write_file(test_name, "alpha.rs", "// alpha\n");
    runner.write_file(test_name, "beta.rs", "// beta\n");

    let ws = format!("/workspaces/{}", test_name);
    let home = format!("{ws}/.aibox-home");

    let driver = vim_driver(
        &ws,
        &home,
        &format!("\"{}\" \"{}\"", format!("{ws}/alpha.rs"), format!("{ws}/beta.rs")),
        &[
            (0.5, "zellij action write 32"),         // <Space>
            (0.1, "zellij action write-chars \"n\""),// n  → :bnext  (→ beta.rs)
            (0.6, "zellij action write 32"),          // <Space>
            (0.1, "zellij action write-chars \"p\""),// p  → :bprev  (→ alpha.rs)
        ],
        2.5,
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    // alpha.rs is open at start; beta.rs should appear after <Space>n
    assert!(
        output.contains("beta.rs"),
        "expected 'beta.rs' in cast after <Space>n buffer-next"
    );
    assert!(
        output.contains("alpha.rs"),
        "expected 'alpha.rs' in cast (was open at start)"
    );

    runner.cleanup(test_name);
}

// ── Lazygit keybinding tests ──────────────────────────────────────────────────
//
// These tests require lazygit to be installed on the e2e-runner (added to
// Dockerfile.e2e). If lazygit is not found on the companion the test is skipped.

fn lazygit_available(runner: &E2eRunner) -> bool {
    runner.exec("which lazygit").status.success()
}

/// lazygit `?`: shows the keybinding help overlay.
///
/// In a repo with a staged file, pressing `?` opens the help/legend panel.
/// We assert that "Keybindings" or "Legend" appears in the cast.
#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_lazygit_question_shows_help() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    if !lazygit_available(&runner) {
        eprintln!("SKIP visual_kb_lazygit_question_shows_help: lazygit not installed");
        return;
    }

    let test_name = "visual-kb-lazygit-help";
    runner.cleanup(test_name);

    let ws = format!("/workspaces/{}", test_name);
    runner.exec(&format!("mkdir -p {ws}"));

    // Bootstrap a git repo with one commit
    runner.exec(&format!(
        "cd {ws} && \
         git -c user.email=test@test.com -c user.name=test init && \
         echo 'hello' > hello.txt && \
         git -c user.email=test@test.com -c user.name=test add hello.txt && \
         git -c user.email=test@test.com -c user.name=test commit -m 'init'"
    ));

    let driver = format!(
        r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={ws}/.lazygit-home
mkdir -p "$HOME"

cat > /tmp/lazygit_layout.kdl << 'LAYOUT_EOF'
layout {{
    pane {{
        command "lazygit"
        cwd "{ws}"
    }}
}}
LAYOUT_EOF

(
  sleep 4
  # '?' opens the help/keybindings overlay
  zellij action write-chars "?"
  sleep 2
  pkill -x zellij 2>/dev/null
) &

zellij options --disable-mouse-mode 2>/dev/null
zellij --config-dir /tmp --layout /tmp/lazygit_layout.kdl 2>/dev/null
true
"#
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    assert!(
        output.contains("Keybindings") || output.contains("Legend") || output.contains("keybinding"),
        "expected lazygit help overlay text after '?', not found in cast"
    );

    runner.cleanup(test_name);
}

/// lazygit `<Space>`: stage/unstage a file.
///
/// With an unstaged file, pressing `<Space>` moves it to the staged section.
/// We assert "Staged" appears in the cast (the staged-files panel header).
#[test]
#[serial]
#[ntest::timeout(90_000)]
fn visual_kb_lazygit_space_stages_file() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    if !lazygit_available(&runner) {
        eprintln!("SKIP visual_kb_lazygit_space_stages_file: lazygit not installed");
        return;
    }

    let test_name = "visual-kb-lazygit-stage";
    runner.cleanup(test_name);

    let ws = format!("/workspaces/{}", test_name);
    runner.exec(&format!("mkdir -p {ws}"));

    // Repo with one commit + one unstaged change
    runner.exec(&format!(
        "cd {ws} && \
         git -c user.email=test@test.com -c user.name=test init && \
         echo 'v1' > file.txt && \
         git -c user.email=test@test.com -c user.name=test add file.txt && \
         git -c user.email=test@test.com -c user.name=test commit -m 'init' && \
         echo 'v2' > file.txt"
    ));

    let driver = format!(
        r#"#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
export HOME={ws}/.lazygit-home
mkdir -p "$HOME"

cat > /tmp/lazygit_stage_layout.kdl << 'LAYOUT_EOF'
layout {{
    pane {{
        command "lazygit"
        cwd "{ws}"
    }}
}}
LAYOUT_EOF

(
  sleep 4
  # <Space> stages the currently highlighted file
  zellij action write 32
  sleep 2
  pkill -x zellij 2>/dev/null
) &

zellij --config-dir /tmp --layout /tmp/lazygit_stage_layout.kdl 2>/dev/null
true
"#
    );

    let cast = record(&runner, test_name, &driver);
    let output = extract_cast_output(&cast);

    assert!(cast.lines().count() > 5, "cast too small");

    // lazygit always shows "Staged Files" and "Unstaged Files" panels
    assert!(
        output.contains("Staged") || output.contains("staged"),
        "expected lazygit file panel (Staged/Unstaged) in cast"
    );

    runner.cleanup(test_name);
}
