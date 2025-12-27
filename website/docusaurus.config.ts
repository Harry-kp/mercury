import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Mercury',
  tagline: 'The last API client you\'ll ever need',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  // Themes and Plugins
  themes: [
    [
      require.resolve('@easyops-cn/docusaurus-search-local'),
      {
        hashed: true,
        language: ['en'],
        highlightSearchTermsOnTargetPage: true,
      },
    ],
  ],

  plugins: [
    'docusaurus-plugin-image-zoom',
  ],

  // GitHub Pages deployment
  url: 'https://harry-kp.github.io',
  baseUrl: '/mercury/',

  organizationName: 'Harry-kp',
  projectName: 'mercury',
  trailingSlash: false,

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/Harry-kp/mercury/tree/main/website/',
        },
        blog: false, // Disable blog
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/mercury-social-card.png',

    // Announcement Bar
    announcementBar: {
      id: 'release_0_2_0',
      content:
        '🚀 Mercury v0.2.0 is out! <a target="_blank" rel="noopener noreferrer" href="https://github.com/Harry-kp/mercury/releases/tag/v0.2.0">Download Now</a>',
      backgroundColor: 'var(--announcement-bg)',
      textColor: 'var(--announcement-text)',
      isCloseable: true,
    },

    // Image Zoom Configuration
    zoom: {
      selector: '.markdown :not(em) > img',
      config: {
        background: {
          light: 'rgb(255, 255, 255)',
          dark: 'rgb(10, 10, 10)'
        }
      }
    },

    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: false,
    },
    navbar: {
      title: 'Mercury',
      logo: {
        alt: 'Mercury Logo',
        src: 'img/logo.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          to: '/docs/quickstart',
          label: 'Quick Start',
          position: 'left',
        },
        {
          href: 'https://github.com/Harry-kp/mercury/releases',
          label: 'Download',
          position: 'right',
        },
        {
          href: 'https://github.com/Harry-kp/mercury',
          label: 'GitHub',
          position: 'right',
        },
        {
          type: 'search',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Learn',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/getting-started',
            },
            {
              label: 'Quick Start',
              to: '/docs/quickstart',
            },
            {
              label: 'Features',
              to: '/docs/features/requests',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'File Format (.http)',
              to: '/docs/reference/file-format',
            },
            {
              label: 'Keyboard Shortcuts',
              to: '/docs/reference/keyboard-shortcuts',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/Harry-kp/mercury',
            },
            {
              label: 'Releases',
              href: 'https://github.com/Harry-kp/mercury/releases',
            },
            {
              label: 'Issues',
              href: 'https://github.com/Harry-kp/mercury/issues',
            },
          ],
        },
      ],
      copyright: `Built with obsessive minimalism by Harry-kp. Open Source under MIT.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'http', 'json', 'toml'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
