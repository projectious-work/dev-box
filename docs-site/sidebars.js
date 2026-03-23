/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    {
      type: 'category', label: 'Getting Started', collapsed: false,
      items: ['getting-started/installation', 'getting-started/new-project', 'getting-started/existing-project'],
    },
    {
      type: 'category', label: 'Container',
      items: ['container/base-image', 'container/addons', 'container/audio'],
    },
    {
      type: 'category', label: 'Context System',
      items: ['context/overview', 'context/process-packages', 'context/migration'],
    },
    {
      type: 'category', label: 'Skills Library',
      items: ['skills/index', 'skills/process', 'skills/development', 'skills/language', 'skills/infrastructure', 'skills/architecture', 'skills/design', 'skills/data', 'skills/ai-ml', 'skills/api', 'skills/security', 'skills/observability', 'skills/database', 'skills/performance', 'skills/framework'],
    },
    {
      type: 'category', label: 'Reference',
      items: ['reference/cli-commands', 'reference/configuration', 'reference/cheatsheet'],
    },
    {
      type: 'category', label: 'Customization',
      items: ['customization/themes'],
    },
    'roadmap',
    {
      type: 'category', label: 'Contributing',
      items: ['contributing/index', 'contributing/maintenance'],
    },
  ],
};

export default sidebars;
