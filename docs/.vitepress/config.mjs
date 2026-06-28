import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "ga4ghphetools",
  description: "Rust library for curating GA4GH Phenopackets",
  themeConfig: {
    nav: [
  { text: 'Home', link: '/' },
  { text: 'Getting Started', link: '/getting-started' },
  {
    text: 'Formats & Output',
    items: [
      { text: 'Phenotype HPOA', link: '/phenotype-hpoa' },
      { text: 'Legacy Templates', link: '/legacy' },
      { text: 'Excel/Output Spec', link: '/excel' }
    ]
  },
  { text: 'API', link: '/api' }
],

    sidebar: [
      {
        text: 'Documentation',
        items: [
          { text: 'Introduction', link: '/introduction' },
          { text: 'Getting Started', link: '/getting-started' },
          { text: 'Phenotype.hpoa', link: '/phenotype-hpoa' },
          { text: 'Legacy Template', link: '/legacy' },
          { text: 'Application', link: '/app' },
          { text: 'Output', link: '/excel' },
          { text: 'API Reference', link: '/api' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/vuejs/vitepress' }
    ]
  }
})
