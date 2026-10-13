# trackscape-discord-ui

Vue 3 + Vite dashboard for TrackScape, styled with Tailwind CSS 4 and daisyUI 5.

## Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur).

## Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) to make the TypeScript language service aware of `.vue` types.

## Customize configuration

See [Vite Configuration Reference](https://vite.dev/config/).

CSS is configured in CSS (Tailwind 4 CSS-first): `src/assets/base.css` holds
`@import "tailwindcss"` and the `@plugin "daisyui"` theme list. There is no
`tailwind.config.js`/`postcss.config.js`.

## Project Setup

Requires Node 24 and [pnpm](https://pnpm.io/) 12 (`packageManager` in
`package.json` pins the version, so Corepack or a matching pnpm works).

```sh
pnpm install
```

### Compile and Hot-Reload for Development

```sh
pnpm run dev
```

### Type-Check, Compile and Minify for Production

```sh
pnpm run build
```

### Run Unit Tests with [Vitest](https://vitest.dev/)

```sh
pnpm run test:unit
```

### Lint with [ESLint](https://eslint.org/)

```sh
pnpm run lint
```
