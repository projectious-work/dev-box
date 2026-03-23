import {themes as prismThemes} from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'dev-box',
  tagline: 'AI-ready development environments, containerized',
  favicon: 'img/favicon.ico',
  url: 'https://projectious-work.github.io',
  baseUrl: '/dev-box/',
  organizationName: 'projectious-work',
  projectName: 'dev-box',
  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',
  i18n: { defaultLocale: 'en', locales: ['en'] },
  markdown: {
    format: 'detect',
  },
  presets: [['classic', {
    docs: {
      sidebarPath: './sidebars.js',
      editUrl: 'https://github.com/projectious-work/dev-box/tree/main/docs-site/',
    },
    blog: false,
    theme: { customCss: './src/css/custom.css' },
  }]],
  themeConfig: {
    colorMode: { defaultMode: 'dark', respectPrefersColorScheme: true },
    navbar: {
      title: 'dev-box',
      items: [
        { type: 'docSidebar', sidebarId: 'docs', position: 'left', label: 'Docs' },
        { to: '/features', label: 'Features', position: 'left' },
        { to: '/changelog', label: 'Changelog', position: 'left' },
        { href: 'https://github.com/projectious-work/dev-box', label: 'GitHub', position: 'right' },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        { title: 'Docs', items: [
          { label: 'Getting Started', to: '/docs/getting-started/installation' },
          { label: 'Configuration', to: '/docs/reference/configuration' },
          { label: 'Skills Library', to: '/docs/skills/' },
        ]},
        { title: 'Project', items: [
          { label: 'GitHub', href: 'https://github.com/projectious-work/dev-box' },
          { label: 'Changelog', to: '/changelog' },
        ]},
      ],
      copyright: `\u00a9 ${new Date().getFullYear()} projectious.work`,
    },
    prism: { theme: prismThemes.github, darkTheme: prismThemes.dracula },
  },
};

export default config;
