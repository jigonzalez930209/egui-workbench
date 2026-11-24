import { defineConfig } from 'vitepress'

// `/` locally; the GitHub Pages workflow sets DOCS_BASE=/egui-workbench/.
const base = process.env.DOCS_BASE ?? '/'

export default defineConfig({
  base,
  lang: 'en-US',
  title: 'egui-workbench',
  description:
    'Reusable desktop workbench shell for egui: themes, title bar, tabs, explorer, dialogs, charts, and an application runtime.',
  cleanUrls: false,
  lastUpdated: true,
  head: [
    ['meta', { name: 'theme-color', content: '#0969da' }],
    ['link', { rel: 'icon', href: `${base}logo.svg` }],
  ],
  markdown: {
    lineNumbers: true,
    theme: { light: 'github-light', dark: 'github-dark' },
  },
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'egui-workbench',
    nav: [
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'Live demo', link: '/guide/live-demo' },
      { text: 'Components', link: '/components/' },
      { text: 'Charts', link: '/charts/overview' },
      { text: 'Theming', link: '/theming/themes' },
      { text: 'Runtime', link: '/app/workbench' },
      { text: 'Reference', link: '/reference/api-map' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Getting started',
          items: [
            { text: 'Introduction', link: '/guide/introduction' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Quick start', link: '/guide/quick-start' },
            { text: 'Build your first app', link: '/guide/first-app' },
            { text: 'Live demo', link: '/guide/live-demo' },
            { text: 'Desktop platforms', link: '/guide/platforms' },
            { text: 'Use with AI agents', link: '/guide/ai-agents' },
          ],
        },
        {
          text: 'Foundations',
          items: [
            { text: 'Architecture', link: '/guide/architecture' },
            { text: 'Core concepts', link: '/guide/concepts' },
          ],
        },
      ],
      '/components/': [
        {
          text: 'Components',
          items: [
            { text: 'Overview', link: '/components/' },
            { text: 'Buttons', link: '/components/buttons' },
            { text: 'Inputs', link: '/components/inputs' },
            { text: 'Display', link: '/components/display' },
            { text: 'Feedback', link: '/components/feedback' },
            { text: 'Menus', link: '/components/menus' },
            { text: 'Overlays', link: '/components/overlays' },
            { text: 'Navigation', link: '/components/navigation' },
            { text: 'Icons', link: '/components/icons' },
            { text: 'Layout', link: '/components/layout' },
            { text: 'Data viewer', link: '/components/data-viewer' },
          ],
        },
      ],
      '/charts/': [
        {
          text: 'Charts',
          items: [
            { text: 'Overview', link: '/charts/overview' },
            { text: 'Series', link: '/charts/series' },
            { text: 'Chart types', link: '/charts/chart-types' },
            { text: '3D charts', link: '/charts/3d' },
            { text: 'Legend animation', link: '/charts/animations' },
            { text: 'Axes and ticks', link: '/charts/axes-and-ticks' },
            { text: 'Interaction', link: '/charts/interaction' },
            { text: 'Configuration', link: '/charts/configuration' },
            { text: 'Fits and smoothing', link: '/charts/fits' },
            { text: 'Export', link: '/charts/export' },
            { text: 'Live data', link: '/charts/live-data' },
            { text: 'Performance', link: '/charts/performance' },
          ],
        },
      ],
      '/theming/': [
        {
          text: 'Theming and customization',
          items: [
            { text: 'Themes', link: '/theming/themes' },
            { text: 'Color tokens', link: '/theming/tokens' },
            { text: 'Size metrics', link: '/theming/metrics' },
            { text: 'Icons', link: '/theming/icons' },
            { text: 'Custom themes', link: '/theming/custom-themes' },
            { text: 'Style and density', link: '/theming/style-and-density' },
            { text: 'Customization layers', link: '/theming/customization' },
          ],
        },
      ],
      '/app/': [
        {
          text: 'Application runtime',
          items: [
            { text: 'Workbench layout', link: '/app/workbench' },
            { text: 'Commands and keymap', link: '/app/commands-and-keymap' },
            { text: 'Documents and close guard', link: '/app/documents' },
            { text: 'History, journal, and sessions', link: '/app/history-and-session' },
            { text: 'Window', link: '/app/window' },
            { text: 'Settings persistence', link: '/app/settings-persistence' },
            { text: 'Internationalization', link: '/app/i18n' },
          ],
        },
      ],
      '/recipes/': [
        {
          text: 'Recipes',
          items: [
            { text: 'Text editor', link: '/recipes/editor' },
            { text: 'Live data panel', link: '/recipes/live-plot' },
            { text: 'Icon browser', link: '/recipes/icon-browser' },
            { text: 'Settings dialog', link: '/recipes/settings-dialog' },
            { text: 'Long-running tasks', link: '/recipes/long-tasks' },
            { text: 'Own brand', link: '/recipes/brand' },
            { text: 'IDE layout', link: '/recipes/ide-layout' },
            { text: 'Responsive content', link: '/recipes/responsive' },
          ],
        },
      ],
      '/reference/': [
        {
          text: 'Reference',
          items: [
            { text: 'API map', link: '/reference/api-map' },
            { text: 'Design rules', link: '/reference/design-rules' },
            { text: 'Testing', link: '/reference/testing' },
            { text: 'Benchmarks', link: '/reference/benchmarks' },
            { text: 'xtask', link: '/reference/xtask' },
            { text: 'Changelog', link: '/reference/changelog' },
            { text: 'Migrating to 0.4', link: '/reference/migration-0.4' },
            { text: 'Roadmap', link: '/reference/roadmap' },
            { text: 'FAQ', link: '/reference/faq' },
          ],
        },
      ],
    },
    search: {
      provider: 'local',
    },
    editLink: {
      pattern: 'https://github.com/jigonzalez930209/egui-workbench/edit/main/docs-site/:path',
      text: 'Edit this page',
    },
    footer: {
      message: 'Released under MIT OR Apache-2.0. Icons: Lucide (ISC) and Material Icon Theme (MIT).',
      copyright: 'egui-workbench',
    },
    docFooter: {
      prev: 'Previous',
      next: 'Next',
    },
    outline: {
      level: [2, 3],
      label: 'On this page',
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/jigonzalez930209/egui-workbench' }],
  },
})
