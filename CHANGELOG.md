# Changelog

## [0.5.0](https://github.com/simplyRoba/gazel/compare/v0.4.0...v0.5.0) (2026-04-09)


### Features

* add charts and visualization with LayerCake ([d9fb94c](https://github.com/simplyRoba/gazel/commit/d9fb94c7fb875165af5d19d4a9a41284ca7e92bc))
* add color contrast with dark slate nav, blue-tinted stat cards, and brand-bg chips ([b1a4dc5](https://github.com/simplyRoba/gazel/commit/b1a4dc5999a6b7cb2b5583046ca5520663d3ca7e))
* show odometer diff, absolute reading, and fuel price on fill-up cards ([e6a5e42](https://github.com/simplyRoba/gazel/commit/e6a5e422356140589355b119004fe9e7e3f2cb1a))


### Bug Fixes

* allow editing non-latest fill-ups by validating against chronological neighbors ([ada3ba1](https://github.com/simplyRoba/gazel/commit/ada3ba1d7ccd59e937900efba0812aed1a6e58ea))
* resolve layercake peer dep conflict in CI ([fb1fc06](https://github.com/simplyRoba/gazel/commit/fb1fc063e047f552056a4233c578c1c885652931))
* sidebar CTA corner triangle should be hover-only like other buttons ([6aed48d](https://github.com/simplyRoba/gazel/commit/6aed48d770575429b7ddd411ede3f1b14a2e90de))

## [0.4.0](https://github.com/simplyRoba/gazel/compare/v0.3.0...v0.4.0) (2026-04-09)


### Features

* add dashboard stats with summary cards, efficiency badges, and stats store ([4e98936](https://github.com/simplyRoba/gazel/commit/4e989366f199a42ce398c5e5746e7474ad052dbe))
* add fill-up CRUD API with validation and vehicle delete guard ([a351de2](https://github.com/simplyRoba/gazel/commit/a351de277a4dd4d19166969382202ac0be9fa298))
* add fill-up logging UI with dashboard integration ([aefc914](https://github.com/simplyRoba/gazel/commit/aefc9143df66dc79f06b09c7d6173097554ba13c))
* add settings and preferences — API, store, formatting utilities, theme server sync ([fe603fd](https://github.com/simplyRoba/gazel/commit/fe603fdf2e42ebba81b1dbf3ef23a0fdabfe7a91))
* add vehicle stats API with efficiency and cost calculations ([50b9dfb](https://github.com/simplyRoba/gazel/commit/50b9dfb7014bffb897c94ccf268af9b4883d0ac4))


### Bug Fixes

* add cancel button to vehicle add/edit forms ([344e816](https://github.com/simplyRoba/gazel/commit/344e816e807079fd216097efd8884b28865d7baa))
* change default currency from USD to EUR ([dff1ed1](https://github.com/simplyRoba/gazel/commit/dff1ed100817a77db0b06c8f73282528778edfaa))
* **deps:** bump tokio from 1.51.0 to 1.51.1 in the tokio group ([8ec36a7](https://github.com/simplyRoba/gazel/commit/8ec36a7a61d7c96aba1bb60a597c0c9796a9038f))
* **deps:** bump vite from 8.0.7 to 8.0.8 in /ui ([d944bb7](https://github.com/simplyRoba/gazel/commit/d944bb735b78a697e78b4ff0cb610f4c1c0df6b4))

## [0.3.0](https://github.com/simplyRoba/gazel/compare/v0.2.0...v0.3.0) (2026-04-08)


### Features

* add CSS component classes — buttons, inputs, chips, badges, cards, skeletons with corner triangle motif ([3eca57f](https://github.com/simplyRoba/gazel/commit/3eca57f1549be6780a660cf465c7327c93a0b29d))
* add ModalDialog and toast notifications — replace inline delete, wire error toasts to vehicle store ([4dd121f](https://github.com/simplyRoba/gazel/commit/4dd121f188cf04fe331d697bf5ee8f81b4a1aa22))
* add vehicle CRUD API — migration, endpoints, validation, PATCH semantics, 22 integration tests ([c3360bb](https://github.com/simplyRoba/gazel/commit/c3360bbf291142ece3d5f7959c5906af17e26383))
* add vehicle management UI — API client, store, form, settings section, dashboard empty state ([327f9dd](https://github.com/simplyRoba/gazel/commit/327f9dd12400b59e54a28207c215910b30ad69e0))
* diamond CTA on mobile bottom bar, remove hover/active background on mobile nav ([0e14182](https://github.com/simplyRoba/gazel/commit/0e141825a32c2d7cdf43049817e24c3975450935))
* refine design language — chamfered corners, triangle accent markers, bold data treatment ([14bcb2b](https://github.com/simplyRoba/gazel/commit/14bcb2b34a7aff0b0a61d241f7e9c343eb85ce51))
* replace chamfered corners with corner triangle motif, add monospace for numbers ([ba883b6](https://github.com/simplyRoba/gazel/commit/ba883b6192da9020635485d6067911d53e1f6a32))
* update logos — sharp corners with accent triangle motif, regenerate all PNGs ([2c2a89e](https://github.com/simplyRoba/gazel/commit/2c2a89e64ddbb47effcc6edfd037c89d55a63724))


### Bug Fixes

* add corner triangles to chips, sidebar nav items, form inputs, and sidebar CTA ([c850fa3](https://github.com/simplyRoba/gazel/commit/c850fa3a8c15c1c3b3213ae34e6c61251674903c))
* apply design language consistently — corner triangles, sharp edges, monospace numbers, chips across all mockups ([20e3770](https://github.com/simplyRoba/gazel/commit/20e377060336f0366b47d6423b55269c24a80d89))
* apply design language to real app — corner triangles on nav/CTA, inline logo mark in mockup ([59570f9](https://github.com/simplyRoba/gazel/commit/59570f9875922cc7ca2d8625ed30969bd15ce8e1))

## [0.2.0](https://github.com/simplyRoba/gazel/compare/v0.1.0...v0.2.0) (2026-04-08)


### Features

* add app shell — responsive layout, navigation with fill-up CTA, theme switching, and base components ([b994d2e](https://github.com/simplyRoba/gazel/commit/b994d2e477f57bb77ecad4dbe1e4a023bd0998cf))
* add backend foundation — Axum server, SQLite, config, API errors, health endpoint, and test harness ([71cc619](https://github.com/simplyRoba/gazel/commit/71cc619c7c99343a88f96096179d2e9c603c87e8))
* add backend foundation — Axum server, SQLite, config, API errors, health endpoint, and test harness ([096e5bd](https://github.com/simplyRoba/gazel/commit/096e5bd3a668572113d59996a634ec6533386069))
* add geometric gazelle logo — SVG source and 512px PNG icon ([0501546](https://github.com/simplyRoba/gazel/commit/050154693b9f6e9c2a6201094d81e7021a264170))
* add light mode icons, dark mode manifest variants, and theme-color meta tags ([92b2d4c](https://github.com/simplyRoba/gazel/commit/92b2d4c234be763601d41d75d24c020e132b5fb6))
* add PWA manifest, favicons, and apple-touch-icon ([27793df](https://github.com/simplyRoba/gazel/commit/27793dff46eabd700eae8543e2737816c37c8743))
* define design system — color palette, typography, spacing, radii, shadows, and CSS tokens ([8fb44f4](https://github.com/simplyRoba/gazel/commit/8fb44f494d8cc5fc787e5f8df5edfa0d409a5894))
* generate favicon and app icon PNGs from logo SVG ([55705da](https://github.com/simplyRoba/gazel/commit/55705da9e2567d46770e934ba148aeb188bf413a))


### Bug Fixes

* **deps:** bump @sveltejs/kit from 2.56.1 to 2.57.0 in /ui in the svelte group ([#3](https://github.com/simplyRoba/gazel/issues/3)) ([ae3dc5e](https://github.com/simplyRoba/gazel/commit/ae3dc5e2d7ec79263c03624221667dc2691a1724))
* **deps:** bump typescript-eslint from 8.58.0 to 8.58.1 in /ui in the eslint group ([#2](https://github.com/simplyRoba/gazel/issues/2)) ([bbf1e6b](https://github.com/simplyRoba/gazel/commit/bbf1e6bf58b4e03392f9022e0d0c8c9c3713d2d1))
* improve color contrast — tertiary text, accent, and semantic colors pass WCAG AA ([acd51e3](https://github.com/simplyRoba/gazel/commit/acd51e3f22076241e60012fb42ad7c8c8e848590))

## 0.1.0 (2026-04-07)


### Features

* add Dockerfile, docker-compose, and configuration env vars ([4e68a29](https://github.com/simplyRoba/gazel/commit/4e68a29e78c515b224af819daa8bc789a04a16bb))
* add minimal rust + sveltekit project scaffold with CI pipelines ([c9d4517](https://github.com/simplyRoba/gazel/commit/c9d451732d94171138de7c3d2a70596ebac2bb60))
