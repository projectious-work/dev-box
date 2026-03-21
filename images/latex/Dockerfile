# =============================================================================
# Stage 1: TeX Live builder — install from CTAN with basic scheme + packages
# =============================================================================
FROM debian:trixie-slim AS texlive-builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    perl \
    wget \
    fontconfig \
    && rm -rf /var/lib/apt/lists/*

# Install TeX Live from CTAN (basic scheme, then add packages)
# Uses a direct CTAN mirror to avoid redirect issues in minimal containers
ARG CTAN_MIRROR=https://ftp.math.utah.edu/pub/tex/historic/systems/texlive/2025/tlnet-final
RUN mkdir -p /tmp/texlive-installer && cd /tmp/texlive-installer \
    && wget -q "${CTAN_MIRROR}/install-tl-unx.tar.gz" \
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
       --repository "${CTAN_MIRROR}" \
    && rm -rf /tmp/texlive-installer

# Symlink all TeX Live binaries into /usr/local/bin (arch-independent)
RUN ln -sf /usr/local/texlive/bin/*/* /usr/local/bin/

# Install LaTeX packages (comprehensive set for academic and technical documents)
# Note: some packages are bundled inside others in TL2025 and not listed separately:
#   amsthm→amscls, array/longtable/multicol→tools, fontenc/graphicx→latex/graphics,
#   nameref→hyperref, pgfplotstable→pgfplots, subcaption→caption, titletoc→titlesec
RUN tlmgr install --no-execute-actions \
    # LuaTeX engine and fonts
    luatex luaotfload luatexbase luacolor lua-ul fontspec unicode-math \
    lualatex-math luahbtex \
    # Core packages
    latex latex-bin latexmk collection-latexrecommended \
    # Page layout and geometry
    geometry fancyhdr lastpage multirow adjustbox \
    # Typography
    microtype parskip setspace titlesec \
    # Tables
    tabularray booktabs tabu colortbl \
    # Colors and graphics
    xcolor graphics epstopdf svg \
    # TikZ/PGF
    pgf tikz-cd tikzmark pgfplots \
    # Boxes
    tcolorbox tikzfill pdfcol environ etoolbox \
    # Bibliography
    biblatex biber csquotes \
    # Code listings
    minted fvextra fancyvrb upquote lineno catchfile xstring framed float \
    # Math
    amsmath amscls mathtools \
    # Hyperlinks and references
    hyperref bookmark cleveref url \
    # Fonts
    lm lm-math sourcecodepro sourcesanspro tex-gyre \
    gnu-freefont \
    # Algorithms
    algorithms algorithmicx algorithm2e \
    # Misc utilities
    caption enumitem listings wrapfig \
    footmisc appendix pdfpages pdflscape xkeyval iftex \
    babel babel-english \
    tools oberdiek amsfonts symbol zapfding \
    kvoptions kvsetkeys kvdefinekeys ltxcmds infwarerr \
    epstopdf-pkg grfext pdftexcmds auxhook intcalc letltxmacro \
    bitset bigintcalc atbegshi atveryend rerunfilecheck \
    uniquecounter refcount gettitlestring hycolor \
    stringenc pdfescape hobsub \
    bytefield siunitx markdown soul ulem \
    todonotes changes datetime2 tracklang \
    was collection-fontsrecommended \
    selnolig emoji \
    # Additional packages (fixes #13)
    ninecolors transparent spath3 nicematrix lipsum \
    && mktexlsr \
    && updmap-sys \
    && fmtutil-sys --all

# Register TeX Live OpenType fonts with fontconfig (fixes #13)
RUN ln -sf /usr/local/texlive/texmf-dist/fonts/opentype /usr/share/fonts/opentype-texlive \
    && fc-cache -f

# =============================================================================
# Stage 2: Runtime — base image + TeX Live tree
# =============================================================================
FROM ghcr.io/projectious-work/dev-box:base-latest AS runtime
# For local builds, use: FROM dev-box-base:latest

ENV DEBIAN_FRONTEND=noninteractive

# Runtime dependencies for TeX Live
RUN apt-get update && apt-get install -y --no-install-recommends \
    perl \
    fontconfig \
    inkscape \
    poppler-utils \
    # latexindent perl dependencies
    libfile-homedir-perl \
    libyaml-tiny-perl \
    liblog-log4perl-perl \
    libunicode-linebreak-perl \
    && rm -rf /var/lib/apt/lists/*

# Copy TeX Live tree from builder and symlink binaries
COPY --from=texlive-builder /usr/local/texlive /usr/local/texlive
RUN ln -sf /usr/local/texlive/bin/*/* /usr/local/bin/

WORKDIR /workspace
CMD ["sleep", "infinity"]
