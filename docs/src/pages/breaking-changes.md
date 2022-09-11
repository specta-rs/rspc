---
title: rspc
layout: ../layouts/MainLayout.astro
---

## 0.0.5 to 0.0.6

All of the frontend code have been split up into multiple npm packages. We now have [`@rspc/client`](https://www.npmjs.com/package/@rspc/client), [`@rspc/react`](https://www.npmjs.com/package/@rspc/react) and [`@rspc/tauri`](https://www.npmjs.com/package/@rspc/tauri). This will help with SSR and reducing project dependencies.

Start by installing the new packages if your require them.

```bash
npm install @rspc/react # If your using the React hooks
npm install @rspc/tauri # If your using the Tauri transport
```

Then change your imports as follows. From:

```ts
import { createReactQueryHooks } from '@rspc/client';
import { TauriTransport } from '@rspc/client';
```

To:

```ts
import { createReactQueryHooks } from '@rspc/react';
import { TauriTransport } from '@rspc/tauri';
```

There is no Rust release for these changes. I am going to look into following SemVer in a future release, this is just a quick patch as multiple people have reported this being an issue with [Next.js](https://nextjs.org) SSR.
