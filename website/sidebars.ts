import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'doc',
      id: 'getting-started',
      label: 'Getting Started',
    },
    {
      type: 'doc',
      id: 'quickstart',
      label: 'Quick Start',
    },
    {
      type: 'doc',
      id: 'performance',
      label: 'Performance',
    },
    {
      type: 'category',
      label: 'Features',
      collapsed: false,
      items: [
        'features/requests',
        'features/collections',
        'features/environments',
        'features/auth',
        'features/history',
        'features/import-export',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/file-format',
        'reference/keyboard-shortcuts',
      ],
    },
    {
      type: 'doc',
      id: 'faq',
      label: 'FAQ',
    },
  ],
};

export default sidebars;
