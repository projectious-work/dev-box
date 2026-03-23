use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Internal recipe definition for an add-on.
/// `addon_version` tracks how *we* install the add-on, not the upstream
/// tool version — that lives in [`ToolDef::supported_versions`].
pub struct AddonDef {
    pub name: &'static str,
    pub addon_version: &'static str,
    pub tools: &'static [ToolDef],
}

/// A single tool inside an add-on with curated version choices.
pub struct ToolDef {
    pub name: &'static str,
    pub default_enabled: bool,
    /// Curated version strings the user can choose from.
    /// Empty slice means no version selection (e.g. clippy, texlive-core).
    pub supported_versions: &'static [&'static str],
    /// Default version string.  `""` when `supported_versions` is empty.
    pub default_version: &'static str,
}

/// Per-tool configuration coming from the parsed `aibox.toml`.
pub struct ToolConfig {
    pub enabled: bool,
    pub version: String,
}

// ---------------------------------------------------------------------------
// Static registry
// ---------------------------------------------------------------------------

static ADDONS: &[AddonDef] = &[
    // ── Language runtimes ────────────────────────────────────────────────
    AddonDef {
        name: "python",
        addon_version: "1.0.0",
        tools: &[
            ToolDef {
                name: "python",
                default_enabled: true,
                supported_versions: &["3.12", "3.13", "3.14"],
                default_version: "3.13",
            },
            ToolDef {
                name: "uv",
                default_enabled: true,
                supported_versions: &["0.6", "0.7"],
                default_version: "0.7",
            },
            ToolDef {
                name: "poetry",
                default_enabled: false,
                supported_versions: &["1.8", "2.0"],
                default_version: "2.0",
            },
            ToolDef {
                name: "pdm",
                default_enabled: false,
                supported_versions: &["2.22"],
                default_version: "2.22",
            },
        ],
    },
    AddonDef {
        name: "rust",
        addon_version: "1.0.0",
        tools: &[
            ToolDef {
                name: "rustc",
                default_enabled: true,
                supported_versions: &["1.85", "1.87"],
                default_version: "1.87",
            },
            ToolDef {
                name: "clippy",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "rustfmt",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
        ],
    },
    AddonDef {
        name: "node",
        addon_version: "1.0.0",
        tools: &[
            ToolDef {
                name: "node",
                default_enabled: true,
                supported_versions: &["20", "22"],
                default_version: "22",
            },
            ToolDef {
                name: "pnpm",
                default_enabled: true,
                supported_versions: &["9", "10"],
                default_version: "10",
            },
            ToolDef {
                name: "yarn",
                default_enabled: false,
                supported_versions: &["4"],
                default_version: "4",
            },
            ToolDef {
                name: "bun",
                default_enabled: false,
                supported_versions: &["1.2"],
                default_version: "1.2",
            },
        ],
    },
    AddonDef {
        name: "go",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "go",
            default_enabled: true,
            supported_versions: &["1.25", "1.26"],
            default_version: "1.26",
        }],
    },
    AddonDef {
        name: "typst",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "typst",
            default_enabled: true,
            supported_versions: &["0.13", "0.14"],
            default_version: "0.14",
        }],
    },
    AddonDef {
        name: "latex",
        addon_version: "1.0.0",
        tools: &[
            ToolDef {
                name: "texlive-core",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-recommended",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-fonts",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-biber",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-code",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-diagrams",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-math",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-music",
                default_enabled: false,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "texlive-chemistry",
                default_enabled: false,
                supported_versions: &[],
                default_version: "",
            },
        ],
    },
    // ── Tool bundles ────────────────────────────────────────────────────
    AddonDef {
        name: "infrastructure",
        addon_version: "1.0.0",
        tools: &[
            ToolDef {
                name: "opentofu",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "ansible",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "packer",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
        ],
    },
    AddonDef {
        name: "kubernetes",
        addon_version: "1.0.0",
        tools: &[
            ToolDef {
                name: "kubectl",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "helm",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "kustomize",
                default_enabled: true,
                supported_versions: &[],
                default_version: "",
            },
            ToolDef {
                name: "k9s",
                default_enabled: false,
                supported_versions: &[],
                default_version: "",
            },
        ],
    },
    AddonDef {
        name: "cloud-aws",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "aws-cli",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "cloud-gcp",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "gcloud-cli",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "cloud-azure",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "azure-cli",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "docs-mkdocs",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "mkdocs",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "docs-zensical",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "zensical",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "docs-docusaurus",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "docusaurus",
            default_enabled: true,
            supported_versions: &["3"],
            default_version: "3",
        }],
    },
    AddonDef {
        name: "docs-starlight",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "starlight",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "docs-mdbook",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "mdbook",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "docs-hugo",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "hugo",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    // ── AI coding agents ────────────────────────────────────────────────
    // Installed per-project via [ai].providers → auto-resolved to addons.
    // See DEC-016: AI providers are addons, not baked into the base image.
    AddonDef {
        name: "ai-claude",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "claude",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "ai-aider",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "aider",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "ai-gemini",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "gemini",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
    AddonDef {
        name: "ai-mistral",
        addon_version: "1.0.0",
        tools: &[ToolDef {
            name: "mistral",
            default_enabled: true,
            supported_versions: &[],
            default_version: "",
        }],
    },
];

// ---------------------------------------------------------------------------
// Lookup functions
// ---------------------------------------------------------------------------

/// Returns the full static list of add-on definitions.
pub fn all_addons() -> &'static [AddonDef] {
    ADDONS
}

/// Look up a single add-on by name.
pub fn get_addon(name: &str) -> Option<&'static AddonDef> {
    ADDONS.iter().find(|a| a.name == name)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check whether a tool is enabled in the user config, falling back to false
/// when the tool is absent from the map.
fn is_enabled(tools: &HashMap<String, ToolConfig>, name: &str) -> bool {
    tools.get(name).is_some_and(|t| t.enabled)
}

/// Retrieve the version string for a tool from user config, falling back to
/// the registry default when missing or empty.
fn version_or_default(
    tools: &HashMap<String, ToolConfig>,
    name: &str,
    default: &str,
) -> String {
    tools
        .get(name)
        .and_then(|t| {
            if t.version.is_empty() {
                None
            } else {
                Some(t.version.clone())
            }
        })
        .unwrap_or_else(|| default.to_string())
}

// ---------------------------------------------------------------------------
// Builder-stage generation
// ---------------------------------------------------------------------------

/// Returns a Dockerfile builder stage block for add-ons that need one.
/// Returns `None` for add-ons that only require runtime commands.
pub fn generate_builder_stage(
    addon_name: &str,
    tools: &HashMap<String, ToolConfig>,
) -> Option<String> {
    match addon_name {
        "rust" => Some(generate_rust_builder(tools)),
        "latex" => Some(generate_latex_builder(tools)),
        "infrastructure" => Some(generate_infrastructure_builder(tools)),
        "kubernetes" => Some(generate_kubernetes_builder(tools)),
        _ => None,
    }
}

fn generate_rust_builder(tools: &HashMap<String, ToolConfig>) -> String {
    let rust_ver = version_or_default(tools, "rustc", "1.87");

    let mut components = Vec::new();
    if is_enabled(tools, "clippy") {
        components.push("clippy");
    }
    if is_enabled(tools, "rustfmt") {
        components.push("rustfmt");
    }
    let component_flags: String = components
        .iter()
        .map(|c| format!(" \\\n    --component {c}"))
        .collect();

    format!(
        r#"# ── Rust builder ─────────────────────────────────────────────────────
FROM debian:trixie-slim AS rust-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
    --default-toolchain {rust_ver}{component_flags}

ENV PATH="/home/aibox/.cargo/bin:${{PATH}}"
"#
    )
}

fn generate_latex_builder(tools: &HashMap<String, ToolConfig>) -> String {
    // tlmgr package groups keyed by tool name
    let mut tlmgr_packages = Vec::new();

    // texlive-core is installed via the basic scheme in the installer itself;
    // additional groups are added via tlmgr.
    if is_enabled(tools, "texlive-recommended") {
        tlmgr_packages.push(
            "    # texlive-recommended\n    \
             latex latex-bin latexmk collection-latexrecommended",
        );
    }
    if is_enabled(tools, "texlive-fonts") {
        tlmgr_packages.push(
            "    # texlive-fonts\n    \
             luatex luaotfload luatexbase luacolor lua-ul fontspec unicode-math \\\n    \
             lualatex-math luahbtex \\\n    \
             lm lm-math sourcecodepro sourcesanspro tex-gyre gnu-freefont \\\n    \
             collection-fontsrecommended selnolig",
        );
    }
    if is_enabled(tools, "texlive-biber") {
        tlmgr_packages.push(
            "    # texlive-biber\n    \
             biblatex biber csquotes",
        );
    }
    if is_enabled(tools, "texlive-code") {
        tlmgr_packages.push(
            "    # texlive-code\n    \
             minted fvextra fancyvrb upquote lineno catchfile xstring framed float \\\n    \
             listings",
        );
    }
    if is_enabled(tools, "texlive-diagrams") {
        tlmgr_packages.push(
            "    # texlive-diagrams\n    \
             pgf tikz-cd tikzmark pgfplots \\\n    \
             tcolorbox tikzfill pdfcol environ etoolbox",
        );
    }
    if is_enabled(tools, "texlive-math") {
        tlmgr_packages.push(
            "    # texlive-math\n    \
             amsmath amscls mathtools amsfonts",
        );
    }
    if is_enabled(tools, "texlive-music") {
        tlmgr_packages.push(
            "    # texlive-music\n    \
             musixtex",
        );
    }
    if is_enabled(tools, "texlive-chemistry") {
        tlmgr_packages.push(
            "    # texlive-chemistry\n    \
             chemfig mhchem",
        );
    }

    // Common packages always installed when latex addon is active
    let common_packages = "\
    # Core layout / typography / tables / colors / hyperlinks / misc\n    \
    geometry fancyhdr lastpage multirow adjustbox \\\n    \
    microtype parskip setspace titlesec \\\n    \
    tabularray booktabs tabu colortbl \\\n    \
    xcolor graphics epstopdf svg \\\n    \
    hyperref bookmark cleveref url \\\n    \
    algorithms algorithmicx algorithm2e \\\n    \
    caption enumitem wrapfig \\\n    \
    footmisc appendix pdfpages pdflscape xkeyval iftex \\\n    \
    babel babel-english \\\n    \
    tools oberdiek symbol zapfding \\\n    \
    kvoptions kvsetkeys kvdefinekeys ltxcmds infwarerr \\\n    \
    epstopdf-pkg grfext pdftexcmds auxhook intcalc letltxmacro \\\n    \
    bitset bigintcalc atbegshi atveryend rerunfilecheck \\\n    \
    uniquecounter refcount gettitlestring hycolor \\\n    \
    stringenc pdfescape hobsub \\\n    \
    bytefield siunitx markdown soul ulem \\\n    \
    todonotes changes datetime2 tracklang was emoji \\\n    \
    ninecolors transparent spath3 nicematrix lipsum";

    let tlmgr_section = if tlmgr_packages.is_empty() {
        common_packages.to_string()
    } else {
        format!(
            "{} \\\n{}",
            common_packages,
            tlmgr_packages.join(" \\\n")
        )
    };

    format!(
        r#"# ── TeX Live builder ──────────────────────────────────────────────────
FROM debian:trixie-slim AS texlive-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    perl \
    wget \
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

ARG CTAN_MIRROR=https://ftp.math.utah.edu/pub/tex/historic/systems/texlive/2025/tlnet-final
RUN mkdir -p /tmp/texlive-installer && cd /tmp/texlive-installer \
    && wget -q "${{CTAN_MIRROR}}/install-tl-unx.tar.gz" \
    && tar -xzf install-tl-unx.tar.gz --strip-components=1 \
    && echo "selected_scheme scheme-basic" > texlive.profile \
    && echo "TEXDIR /usr/local/texlive" >> texlive.profile \
    && echo "TEXMFLOCAL /usr/local/texlive/texmf-local" >> texlive.profile \
    && echo "TEXMFSYSCONFIG /usr/local/texlive/texmf-config" >> texlive.profile \
    && echo "TEXMFSYSVAR /usr/local/texlive/texmf-var" >> texlive.profile \
    && echo "option_doc 0" >> texlive.profile \
    && echo "option_src 0" >> texlive.profile \
    && echo "tlpdbopt_autobackup 0" >> texlive.profile \
    && echo "tlpdbopt_install_docfiles 0" >> texlive.profile \
    && echo "tlpdbopt_install_srcfiles 0" >> texlive.profile \
    && ./install-tl --profile=texlive.profile --no-interaction \
       --repository "${{CTAN_MIRROR}}" \
    && rm -rf /tmp/texlive-installer \
    && ln -sf /usr/local/texlive/bin/*/* /usr/local/bin/

RUN tlmgr install --no-execute-actions \
    {tlmgr_section} \
    && mktexlsr \
    && updmap-sys \
    && fmtutil-sys --all
"#
    )
}

fn generate_infrastructure_builder(tools: &HashMap<String, ToolConfig>) -> String {
    let mut commands = Vec::new();

    if is_enabled(tools, "opentofu") {
        commands.push(
            "    curl -fsSL https://get.opentofu.org/install-opentofu.sh \
| sh -s -- --install-method standalone --install-path /build/bin"
                .to_string(),
        );
    }
    if is_enabled(tools, "packer") {
        commands.push(
            "    ARCH=\"$(dpkg --print-architecture)\" && \\\n    \
             curl -fsSL \"https://releases.hashicorp.com/packer/1.11.2/packer_1.11.2_linux_${ARCH}.zip\" \
-o /tmp/packer.zip && \\\n    \
             unzip -q /tmp/packer.zip -d /build/bin && rm /tmp/packer.zip"
                .to_string(),
        );
    }

    if commands.is_empty() {
        return String::new();
    }

    format!(
        r#"# ── Infrastructure builder ────────────────────────────────────────────
FROM debian:trixie-slim AS infra-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /build/bin && \
{cmds}
"#,
        cmds = commands.join(" && \\\n")
    )
}

fn generate_kubernetes_builder(tools: &HashMap<String, ToolConfig>) -> String {
    let mut commands = Vec::new();

    if is_enabled(tools, "kubectl") {
        commands.push(
            "    ARCH=\"$(dpkg --print-architecture)\" && \\\n    \
             curl -fsSL \"https://dl.k8s.io/release/$(curl -fsSL \
https://dl.k8s.io/release/stable.txt)/bin/linux/${ARCH}/kubectl\" \
-o /build/bin/kubectl && \\\n    \
             chmod +x /build/bin/kubectl"
                .to_string(),
        );
    }
    if is_enabled(tools, "helm") {
        commands.push(
            "    curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 \
| HELM_INSTALL_DIR=/build/bin bash"
                .to_string(),
        );
    }
    if is_enabled(tools, "kustomize") {
        commands.push(
            "    curl -fsSL \"https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh\" \
| bash && \\\n    \
             mv kustomize /build/bin/"
                .to_string(),
        );
    }
    if is_enabled(tools, "k9s") {
        commands.push(
            "    ARCH=\"$(dpkg --print-architecture)\" && \\\n    \
             curl -fsSL \"https://github.com/derailed/k9s/releases/latest/download/k9s_Linux_${ARCH}.tar.gz\" \
| tar xz -C /build/bin k9s"
                .to_string(),
        );
    }

    if commands.is_empty() {
        return String::new();
    }

    format!(
        r#"# ── Kubernetes builder ────────────────────────────────────────────────
FROM debian:trixie-slim AS k8s-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /build/bin && \
{cmds}
"#,
        cmds = commands.join(" && \\\n")
    )
}

// ---------------------------------------------------------------------------
// Runtime-command generation
// ---------------------------------------------------------------------------

/// Returns Dockerfile `RUN` commands for the runtime stage of a given add-on.
pub fn generate_runtime_commands(
    addon_name: &str,
    tools: &HashMap<String, ToolConfig>,
) -> String {
    match addon_name {
        "python" => generate_python_runtime(tools),
        "rust" => generate_rust_runtime(tools),
        "node" => generate_node_runtime(tools),
        "go" => generate_go_runtime(tools),
        "typst" => generate_typst_runtime(tools),
        "latex" => generate_latex_runtime(tools),
        "infrastructure" => generate_infrastructure_runtime(tools),
        "kubernetes" => generate_kubernetes_runtime(tools),
        "cloud-aws" => generate_cloud_aws_runtime(tools),
        "cloud-gcp" => generate_cloud_gcp_runtime(tools),
        "cloud-azure" => generate_cloud_azure_runtime(tools),
        "docs-mkdocs" => generate_docs_mkdocs_runtime(tools),
        "docs-zensical" => generate_docs_zensical_runtime(tools),
        "docs-docusaurus" => generate_docs_docusaurus_runtime(tools),
        "docs-starlight" => generate_docs_starlight_runtime(tools),
        "docs-mdbook" => generate_docs_mdbook_runtime(tools),
        "docs-hugo" => generate_docs_hugo_runtime(tools),
        "ai-claude" => generate_ai_claude_runtime(tools),
        "ai-aider" => generate_ai_aider_runtime(tools),
        "ai-gemini" => generate_ai_gemini_runtime(tools),
        "ai-mistral" => generate_ai_mistral_runtime(tools),
        _ => String::new(),
    }
}

// ── Python ──────────────────────────────────────────────────────────────

fn generate_python_runtime(tools: &HashMap<String, ToolConfig>) -> String {
    let mut parts = Vec::new();
    let py_ver = version_or_default(tools, "python", "3.13");

    // Python runtime — the system package version tracks the major.minor from
    // the Debian repository.  For exact version pinning a PPA/deadsnakes would
    // be needed, but Debian trixie ships 3.13 which is the current default.
    parts.push(format!(
        "# Addon: python (runtime)\n\
         RUN apt-get update && apt-get install -y --no-install-recommends \\\n    \
             python{py_ver} \\\n    \
             python3-pip \\\n    \
             python3-venv \\\n    \
             && rm -rf /var/lib/apt/lists/*"
    ));

    if is_enabled(tools, "uv") {
        let uv_ver = version_or_default(tools, "uv", "0.7");
        parts.push(format!(
            "COPY --from=ghcr.io/astral-sh/uv:{uv_ver} /uv /usr/local/bin/uv\n\
             COPY --from=ghcr.io/astral-sh/uv:{uv_ver} /uvx /usr/local/bin/uvx"
        ));
    }

    if is_enabled(tools, "poetry") {
        let poetry_ver = version_or_default(tools, "poetry", "2.0");
        parts.push(format!(
            "RUN pip3 install --no-cache-dir 'poetry~={poetry_ver}.0'"
        ));
    }

    if is_enabled(tools, "pdm") {
        let pdm_ver = version_or_default(tools, "pdm", "2.22");
        parts.push(format!(
            "RUN pip3 install --no-cache-dir 'pdm~={pdm_ver}.0'"
        ));
    }

    parts.push("ENV PATH=\"/home/aibox/.local/bin:${PATH}\"".to_string());
    parts.join("\n\n")
}

// ── Rust ────────────────────────────────────────────────────────────────

fn generate_rust_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    // The heavy lifting happens in the builder stage.  The runtime stage
    // copies the installed toolchain over.
    "# Addon: rust (runtime — copy from builder)\n\
     COPY --from=rust-builder /home/aibox/.cargo /home/aibox/.cargo\n\
     COPY --from=rust-builder /home/aibox/.rustup /home/aibox/.rustup\n\n\
     ENV PATH=\"/home/aibox/.cargo/bin:${PATH}\""
        .to_string()
}

// ── Node ────────────────────────────────────────────────────────────────

fn generate_node_runtime(tools: &HashMap<String, ToolConfig>) -> String {
    let node_ver = version_or_default(tools, "node", "22");
    let mut parts = Vec::new();

    parts.push(format!(
        "# Addon: node (runtime)\n\
         RUN curl -fsSL https://deb.nodesource.com/setup_{node_ver}.x | bash - && \\\n    \
             apt-get install -y --no-install-recommends nodejs && \\\n    \
             rm -rf /var/lib/apt/lists/*"
    ));

    let mut npm_globals = Vec::new();
    if is_enabled(tools, "pnpm") {
        let pnpm_ver = version_or_default(tools, "pnpm", "10");
        npm_globals.push(format!("pnpm@{pnpm_ver}"));
    }
    if is_enabled(tools, "yarn") {
        let yarn_ver = version_or_default(tools, "yarn", "4");
        npm_globals.push(format!("yarn@{yarn_ver}"));
    }
    if !npm_globals.is_empty() {
        parts.push(format!(
            "RUN npm install -g {}",
            npm_globals.join(" ")
        ));
    }

    if is_enabled(tools, "bun") {
        let bun_ver = version_or_default(tools, "bun", "1.2");
        parts.push(format!(
            "RUN curl -fsSL https://bun.sh/install | BUN_INSTALL=/usr/local bash -s -- \"bun-v{bun_ver}\""
        ));
    }

    parts.join("\n\n")
}

// ── Go ──────────────────────────────────────────────────────────────────

fn generate_go_runtime(tools: &HashMap<String, ToolConfig>) -> String {
    let go_ver = version_or_default(tools, "go", "1.26");
    format!(
        "# Addon: go (runtime)\n\
         RUN ARCH=$(dpkg --print-architecture) && \\\n    \
             curl -fsSL \"https://go.dev/dl/go{go_ver}.linux-${{ARCH}}.tar.gz\" \\\n    \
             | tar -xz -C /usr/local && \\\n    \
             ln -sf /usr/local/go/bin/* /usr/local/bin/\n\n\
         ENV GOPATH=\"/home/aibox/go\"\n\
         ENV PATH=\"${{GOPATH}}/bin:${{PATH}}\""
    )
}

// ── Typst ───────────────────────────────────────────────────────────────

fn generate_typst_runtime(tools: &HashMap<String, ToolConfig>) -> String {
    let typst_ver = version_or_default(tools, "typst", "0.14");
    format!(
        "# Addon: typst (runtime)\n\
         RUN apt-get update && apt-get install -y --no-install-recommends xz-utils \\\n    \
             && rm -rf /var/lib/apt/lists/* \\\n    \
             && ARCH=$(uname -m) \\\n    \
             && curl -fsSL \\\n      \
               \"https://github.com/typst/typst/releases/download/v{typst_ver}/typst-${{ARCH}}-unknown-linux-musl.tar.xz\" \\\n      \
               | tar -xJ --strip-components=1 -C /usr/local/bin \\\n        \
                 \"typst-${{ARCH}}-unknown-linux-musl/typst\" \\\n    \
             && chmod +x /usr/local/bin/typst"
    )
}

// ── LaTeX ───────────────────────────────────────────────────────────────

fn generate_latex_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    // Runtime dependencies + copy TeX Live tree from builder.
    "# Addon: latex (runtime — copy from builder)\n\
     RUN apt-get update && apt-get install -y --no-install-recommends \\\n    \
         perl \\\n    \
         fontconfig \\\n    \
         inkscape \\\n    \
         poppler-utils \\\n    \
         libfile-homedir-perl \\\n    \
         libyaml-tiny-perl \\\n    \
         liblog-log4perl-perl \\\n    \
         libunicode-linebreak-perl \\\n    \
         && rm -rf /var/lib/apt/lists/*\n\n\
     COPY --from=texlive-builder /usr/local/texlive /usr/local/texlive\n\
     RUN ln -sf /usr/local/texlive/bin/*/* /usr/local/bin/ && \\\n    \
         ln -sf /usr/local/texlive/texmf-dist/fonts/opentype /usr/share/fonts/opentype-texlive && \\\n    \
         fc-cache -f"
        .to_string()
}

// ── Infrastructure ──────────────────────────────────────────────────────

fn generate_infrastructure_runtime(tools: &HashMap<String, ToolConfig>) -> String {
    let mut parts = Vec::new();
    parts.push("# Addon: infrastructure (runtime)".to_string());

    // Binaries from builder
    let mut copy_bins = Vec::new();
    if is_enabled(tools, "opentofu") {
        copy_bins.push("tofu");
    }
    if is_enabled(tools, "packer") {
        copy_bins.push("packer");
    }
    if !copy_bins.is_empty() {
        for bin in &copy_bins {
            parts.push(format!(
                "COPY --from=infra-builder /build/bin/{bin} /usr/local/bin/{bin}"
            ));
        }
    }

    // Ansible is pip-installed at runtime (not a static binary)
    if is_enabled(tools, "ansible") {
        parts.push("RUN pip3 install --no-cache-dir ansible".to_string());
    }

    parts.join("\n")
}

// ── Kubernetes ──────────────────────────────────────────────────────────

fn generate_kubernetes_runtime(tools: &HashMap<String, ToolConfig>) -> String {
    let mut parts = Vec::new();
    parts.push("# Addon: kubernetes (runtime)".to_string());

    let bins: Vec<&str> = ["kubectl", "helm", "kustomize", "k9s"]
        .iter()
        .copied()
        .filter(|b| is_enabled(tools, b))
        .collect();

    for bin in &bins {
        parts.push(format!(
            "COPY --from=k8s-builder /build/bin/{bin} /usr/local/bin/{bin}"
        ));
    }

    parts.join("\n")
}

// ── Cloud providers ─────────────────────────────────────────────────────

fn generate_cloud_aws_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: cloud-aws\n\
     RUN ARCH=\"$(uname -m)\" && \\\n    \
         curl -fsSL \"https://awscli.amazonaws.com/awscli-exe-linux-${ARCH}.zip\" -o /tmp/awscli.zip && \\\n    \
         unzip -q /tmp/awscli.zip -d /tmp && \\\n    \
         /tmp/aws/install && \\\n    \
         rm -rf /tmp/aws /tmp/awscli.zip"
        .to_string()
}

fn generate_cloud_gcp_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: cloud-gcp\n\
     RUN curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg \
| gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg && \\\n    \
         echo \"deb [signed-by=/usr/share/keyrings/cloud.google.gpg] \
https://packages.cloud.google.com/apt cloud-sdk main\" > /etc/apt/sources.list.d/google-cloud-sdk.list && \\\n    \
         apt-get update && apt-get install -y --no-install-recommends google-cloud-cli && \\\n    \
         rm -rf /var/lib/apt/lists/*"
        .to_string()
}

fn generate_cloud_azure_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: cloud-azure\n\
     RUN pip3 install --no-cache-dir azure-cli"
        .to_string()
}

// ── Docs ────────────────────────────────────────────────────────────────

fn generate_docs_mkdocs_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: docs-mkdocs\n\
     RUN uv tool install 'mkdocs<2' --with mkdocs-material"
        .to_string()
}

fn generate_docs_zensical_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: docs-zensical\n\
     RUN uv tool install zensical"
        .to_string()
}

fn generate_docs_docusaurus_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: docs-docusaurus\n\
     RUN npm install -g docusaurus"
        .to_string()
}

fn generate_docs_starlight_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: docs-starlight\n\
     RUN npm install -g create-starlight"
        .to_string()
}

fn generate_docs_mdbook_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: docs-mdbook\n\
     RUN ARCH=\"$(uname -m)\" && \\\n    \
         curl -fsSL \"https://github.com/rust-lang/mdBook/releases/latest/download/mdbook-v0.4.43-${ARCH}-unknown-linux-musl.tar.gz\" \\\n    \
         | tar -xz -C /usr/local/bin"
        .to_string()
}

fn generate_docs_hugo_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: docs-hugo\n\
     RUN ARCH=\"$(dpkg --print-architecture)\" && \\\n    \
         curl -fsSL \"https://github.com/gohugoio/hugo/releases/latest/download/hugo_extended_0.141.0_linux-${ARCH}.tar.gz\" \\\n    \
         | tar -xz -C /usr/local/bin hugo"
        .to_string()
}

// ── AI coding agents ─────────────────────────────────────────────────

fn generate_ai_claude_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: ai-claude\n\
     USER aibox\n\
     RUN curl -fsSL https://claude.ai/install.sh | bash\n\
     USER root"
        .to_string()
}

fn generate_ai_aider_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: ai-aider\n\
     RUN uv tool install aider-chat"
        .to_string()
}

fn generate_ai_gemini_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: ai-gemini\n\
     RUN npm install -g @google/generative-ai-cli || pip install google-generativeai"
        .to_string()
}

fn generate_ai_mistral_runtime(_tools: &HashMap<String, ToolConfig>) -> String {
    "# Addon: ai-mistral\n\
     RUN pip install --no-cache-dir mistralai"
        .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a tool config map from (name, enabled, version) tuples.
    fn tc(entries: &[(&str, bool, &str)]) -> HashMap<String, ToolConfig> {
        entries
            .iter()
            .map(|(n, e, v)| {
                (
                    n.to_string(),
                    ToolConfig {
                        enabled: *e,
                        version: v.to_string(),
                    },
                )
            })
            .collect()
    }

    // ── Registry lookup ─────────────────────────────────────────────────

    #[test]
    fn all_addons_returns_all_entries() {
        let addons = all_addons();
        assert!(addons.len() >= 21, "expected at least 21 add-ons (17 tools + 4 AI agents)");
    }

    #[test]
    fn get_addon_finds_known() {
        assert!(get_addon("python").is_some());
        assert!(get_addon("rust").is_some());
        assert!(get_addon("latex").is_some());
        assert!(get_addon("kubernetes").is_some());
        assert!(get_addon("cloud-aws").is_some());
        assert!(get_addon("docs-zensical").is_some());
    }

    #[test]
    fn get_addon_returns_none_for_unknown() {
        assert!(get_addon("doesnotexist").is_none());
    }

    // ── Version validation ──────────────────────────────────────────────

    #[test]
    fn python_tool_versions_are_curated() {
        let addon = get_addon("python").unwrap();
        let py = addon.tools.iter().find(|t| t.name == "python").unwrap();
        assert!(py.supported_versions.contains(&"3.13"));
        assert_eq!(py.default_version, "3.13");
    }

    #[test]
    fn rust_tool_versions_are_curated() {
        let addon = get_addon("rust").unwrap();
        let rustc = addon.tools.iter().find(|t| t.name == "rustc").unwrap();
        assert!(rustc.supported_versions.contains(&"1.87"));
        assert_eq!(rustc.default_version, "1.87");
    }

    #[test]
    fn versionless_tools_have_empty_defaults() {
        let addon = get_addon("rust").unwrap();
        let clippy = addon.tools.iter().find(|t| t.name == "clippy").unwrap();
        assert!(clippy.supported_versions.is_empty());
        assert_eq!(clippy.default_version, "");
    }

    // ── Builder-stage generation ────────────────────────────────────────

    #[test]
    fn rust_has_builder_stage() {
        let tools = tc(&[
            ("rustc", true, "1.87"),
            ("clippy", true, ""),
            ("rustfmt", true, ""),
        ]);
        let stage = generate_builder_stage("rust", &tools);
        assert!(stage.is_some());
        let stage = stage.unwrap();
        assert!(stage.contains("rust-builder"));
        assert!(stage.contains("1.87"));
        assert!(stage.contains("--component clippy"));
        assert!(stage.contains("--component rustfmt"));
    }

    #[test]
    fn latex_has_builder_stage() {
        let tools = tc(&[
            ("texlive-core", true, ""),
            ("texlive-recommended", true, ""),
            ("texlive-fonts", true, ""),
            ("texlive-biber", true, ""),
            ("texlive-code", true, ""),
            ("texlive-diagrams", true, ""),
            ("texlive-math", true, ""),
        ]);
        let stage = generate_builder_stage("latex", &tools);
        assert!(stage.is_some());
        let stage = stage.unwrap();
        assert!(stage.contains("texlive-builder"));
        assert!(stage.contains("tlmgr install"));
        assert!(stage.contains("biblatex"));
        assert!(stage.contains("pgf"));
    }

    #[test]
    fn kubernetes_has_builder_stage() {
        let tools = tc(&[
            ("kubectl", true, ""),
            ("helm", true, ""),
            ("kustomize", true, ""),
        ]);
        let stage = generate_builder_stage("kubernetes", &tools);
        assert!(stage.is_some());
        let stage = stage.unwrap();
        assert!(stage.contains("k8s-builder"));
        assert!(stage.contains("kubectl"));
    }

    #[test]
    fn python_has_no_builder_stage() {
        let tools = tc(&[("python", true, "3.13"), ("uv", true, "0.7")]);
        assert!(generate_builder_stage("python", &tools).is_none());
    }

    // ── Runtime-command generation ──────────────────────────────────────

    #[test]
    fn python_runtime_contains_uv() {
        let tools = tc(&[("python", true, "3.13"), ("uv", true, "0.7")]);
        let cmds = generate_runtime_commands("python", &tools);
        assert!(cmds.contains("python3.13") || cmds.contains("python3"));
        assert!(cmds.contains("astral-sh/uv:0.7"));
    }

    #[test]
    fn node_runtime_with_pnpm() {
        let tools = tc(&[("node", true, "22"), ("pnpm", true, "10")]);
        let cmds = generate_runtime_commands("node", &tools);
        assert!(cmds.contains("setup_22.x"));
        assert!(cmds.contains("pnpm@10"));
    }

    #[test]
    fn go_runtime_uses_version() {
        let tools = tc(&[("go", true, "1.25")]);
        let cmds = generate_runtime_commands("go", &tools);
        assert!(cmds.contains("go1.25.linux"));
    }

    #[test]
    fn typst_runtime_uses_version() {
        let tools = tc(&[("typst", true, "0.13")]);
        let cmds = generate_runtime_commands("typst", &tools);
        assert!(cmds.contains("/v0.13/"));
    }

    #[test]
    fn cloud_aws_runtime() {
        let tools = tc(&[("aws-cli", true, "")]);
        let cmds = generate_runtime_commands("cloud-aws", &tools);
        assert!(cmds.contains("awscli"));
    }

    #[test]
    fn cloud_gcp_runtime() {
        let tools = tc(&[("gcloud-cli", true, "")]);
        let cmds = generate_runtime_commands("cloud-gcp", &tools);
        assert!(cmds.contains("google-cloud-cli"));
    }

    #[test]
    fn cloud_azure_runtime() {
        let tools = tc(&[("azure-cli", true, "")]);
        let cmds = generate_runtime_commands("cloud-azure", &tools);
        assert!(cmds.contains("azure-cli"));
    }

    #[test]
    fn docs_mkdocs_runtime() {
        let tools = tc(&[("mkdocs", true, "")]);
        let cmds = generate_runtime_commands("docs-mkdocs", &tools);
        assert!(cmds.contains("mkdocs"));
        assert!(cmds.contains("mkdocs-material"));
    }

    #[test]
    fn docs_zensical_runtime() {
        let tools = tc(&[("zensical", true, "")]);
        let cmds = generate_runtime_commands("docs-zensical", &tools);
        assert!(cmds.contains("zensical"));
    }

    #[test]
    fn docs_hugo_runtime() {
        let tools = tc(&[("hugo", true, "")]);
        let cmds = generate_runtime_commands("docs-hugo", &tools);
        assert!(cmds.contains("hugo"));
    }

    #[test]
    fn docs_mdbook_runtime() {
        let tools = tc(&[("mdbook", true, "")]);
        let cmds = generate_runtime_commands("docs-mdbook", &tools);
        assert!(cmds.contains("mdbook") || cmds.contains("mdBook"));
    }

    #[test]
    fn unknown_addon_returns_empty_runtime() {
        let tools = HashMap::new();
        let cmds = generate_runtime_commands("nonexistent", &tools);
        assert!(cmds.is_empty());
    }

    #[test]
    fn rust_runtime_copies_from_builder() {
        let tools = tc(&[("rustc", true, "1.87")]);
        let cmds = generate_runtime_commands("rust", &tools);
        assert!(cmds.contains("COPY --from=rust-builder"));
    }

    #[test]
    fn latex_runtime_copies_from_builder() {
        let tools = tc(&[("texlive-core", true, "")]);
        let cmds = generate_runtime_commands("latex", &tools);
        assert!(cmds.contains("COPY --from=texlive-builder"));
    }

    #[test]
    fn kubernetes_runtime_copies_from_builder() {
        let tools = tc(&[("kubectl", true, ""), ("helm", true, "")]);
        let cmds = generate_runtime_commands("kubernetes", &tools);
        assert!(cmds.contains("COPY --from=k8s-builder"));
        assert!(cmds.contains("kubectl"));
        assert!(cmds.contains("helm"));
    }

    #[test]
    fn infrastructure_runtime_copies_binaries() {
        let tools = tc(&[
            ("opentofu", true, ""),
            ("packer", true, ""),
            ("ansible", true, ""),
        ]);
        let cmds = generate_runtime_commands("infrastructure", &tools);
        assert!(cmds.contains("COPY --from=infra-builder"));
        assert!(cmds.contains("ansible"));
    }

    // ── Default-enabled / default-disabled ──────────────────────────────

    #[test]
    fn default_enabled_flags_match_spec() {
        let python = get_addon("python").unwrap();
        let poetry = python.tools.iter().find(|t| t.name == "poetry").unwrap();
        assert!(!poetry.default_enabled, "poetry should be off by default");

        let node = get_addon("node").unwrap();
        let bun = node.tools.iter().find(|t| t.name == "bun").unwrap();
        assert!(!bun.default_enabled, "bun should be off by default");

        let k8s = get_addon("kubernetes").unwrap();
        let k9s = k8s.tools.iter().find(|t| t.name == "k9s").unwrap();
        assert!(!k9s.default_enabled, "k9s should be off by default");

        let latex = get_addon("latex").unwrap();
        let music = latex
            .tools
            .iter()
            .find(|t| t.name == "texlive-music")
            .unwrap();
        assert!(!music.default_enabled, "texlive-music should be off by default");
    }

    #[test]
    fn node_bun_optional_included_when_enabled() {
        let tools = tc(&[
            ("node", true, "22"),
            ("pnpm", true, "10"),
            ("bun", true, "1.2"),
        ]);
        let cmds = generate_runtime_commands("node", &tools);
        assert!(cmds.contains("bun"));
    }

    // ── AI provider addons ──────────────────────────────────────────────

    #[test]
    fn ai_claude_addon_exists() {
        assert!(get_addon("ai-claude").is_some());
    }

    #[test]
    fn ai_claude_runtime_installs_claude() {
        let tools = tc(&[("claude", true, "")]);
        let cmds = generate_runtime_commands("ai-claude", &tools);
        assert!(cmds.contains("claude.ai/install.sh"), "should install Claude Code CLI");
        assert!(cmds.contains("USER aibox"), "should switch to aibox user for install");
    }

    #[test]
    fn ai_aider_runtime_installs_aider() {
        let tools = tc(&[("aider", true, "")]);
        let cmds = generate_runtime_commands("ai-aider", &tools);
        assert!(cmds.contains("aider-chat"), "should install aider via uv");
    }

    #[test]
    fn ai_gemini_runtime_installs_gemini() {
        let tools = tc(&[("gemini", true, "")]);
        let cmds = generate_runtime_commands("ai-gemini", &tools);
        assert!(cmds.contains("generative-ai"), "should install gemini CLI");
    }

    #[test]
    fn ai_mistral_runtime_installs_mistral() {
        let tools = tc(&[("mistral", true, "")]);
        let cmds = generate_runtime_commands("ai-mistral", &tools);
        assert!(cmds.contains("mistralai"), "should install mistral SDK");
    }
}
