import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Snakebots",
  description: "A game about programming snakes and competing against other players",
  themeConfig: {
    logo: '/assets/icon.svg',

    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Language', link: '/language/specification', activeMatch: '/language' },
      { text: 'Game', link: '/game/specification', activeMatch: '/game' },
    ],

    sidebar: [
      {
        text: 'Introduction',
        items: [
          { text: 'Getting Started', link: '/getting-started' },
        ]
      },
      {
        text: 'Language',
        items: [
          { text: 'Specification', link: '/language/specification' },
        ]
      },
      {
        text: 'Game',
        items: [
          { text: 'Specification', link: '/game/specification' },
        ]
      },
    ],

    // socialLinks: [
    //   { icon: 'github', link: 'https://github.com/vuejs/vitepress' }
    // ]
  },
  vite: {
    build: {
      assetsInlineLimit: 0,
    }
  },
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico', sizes: 'any' }],
    ['link', { rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' }],
  ]
})
