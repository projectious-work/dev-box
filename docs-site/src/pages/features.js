import React from 'react';
import Layout from '@theme/Layout';

export default function Features() {
  return (
    <Layout title="Features" description="dev-box features overview">
      <div className="container margin-vert--lg">
        <h1>Features</h1>

        <h2>Single Base Image + Add-ons</h2>
        <p>One base-debian image provides the foundation. Everything else — Python, Rust, Node, LaTeX, Kubernetes tools — is a declarative add-on with per-tool version selection.</p>

        <h2>13 Composable Process Packages</h2>
        <p>No more one-size-fits-all. Pick the packages that match your project: tracking, standups, handover, code, research, documentation, design, architecture, security, data, operations, and product management.</p>

        <h2>83 Curated AI Skills</h2>
        <p>Each skill follows the SKILL.md standard with progressive disclosure: trigger conditions, structured instructions, and real examples. Skills are selectively deployed based on your process packages.</p>

        <h2>6 Color Themes</h2>
        <p>Gruvbox Dark, Catppuccin Mocha, Catppuccin Latte, Dracula, Tokyo Night, and Nord — applied consistently across Zellij, Vim, Yazi, lazygit, and Starship.</p>

        <h2>Declarative Configuration</h2>
        <p>One dev-box.toml controls everything: base image, add-ons with versions, process packages, skill selection, themes, and AI providers. Run dev-box sync to reconcile.</p>

        <h2>Migration System</h2>
        <p>When dev-box updates, migration documents are auto-generated with safety headers, action items, and verification checklists. Your AI agent picks them up at session start.</p>
      </div>
    </Layout>
  );
}
