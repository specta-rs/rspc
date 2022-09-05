---
title: rspc
layout: ../layouts/MainLayout.astro
---

## 0.0.5 to 0.0.6

All of the frontend code have been split up into multiple packages. We now have `@rspc/client`, `@rspc/react` and `@rspc/tauri`. This will help with SSR and reducing project dependencies. You must change imports as follows.

From:

```ts
import { createReactQueryHooks } from '@rspc/client';
import { TauriTransport } from '@rspc/client';
```

To:

```ts
import { createReactQueryHooks } from '@rspc/react';
import { TauriTransport } from '@rspc/tauri';
```

There is no Rust release for these changes. I am going to look into following SemVer in a future release, this is just a quick patch as multiple people have reported it.
