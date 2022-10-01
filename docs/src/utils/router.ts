// This file is pretty disgusting but I just wanted to throw together a working prototype of single page routing with Astro.

import morphdom from "morphdom";

declare global {
  interface Window {
    cacheId: string | undefined;
  }
}

const pageSelector = "#page";
const page = document.querySelector(pageSelector);
if (page === null) throw new Error("Unable to find root page element.");
if (window.cacheId === undefined)
  throw new Error("CacheId is not defined on the window");
const cacheName = `spa-cache-${window.cacheId || ""}`;
const cache = await caches.open(cacheName);
const pagesBeingFetched = new Map<string, Promise<undefined>>();
let isNavigating = false;
const domParser = new DOMParser();

caches.keys().then((c) =>
  c.map((c) => {
    if (c !== cacheName) caches.delete(c);
  })
);

// Mobile navigation bar handler
const button = document.querySelector("#sidebar-toggle");
const body = document.querySelector("body");
function toggleMobileNav() {
  body?.classList.toggle("mobile-sidebar-toggle");
  button?.toggleAttribute("aria-pressed");
}

// Handle change page to a specific URL
async function navigate(url: URL, force?: boolean) {
  if (force !== true && url.pathname === window.location.pathname) return;

  if (pagesBeingFetched.has(url.toString())) {
    await pagesBeingFetched.get(url.toString());
  }

  let resp = await cache.match(url);
  if (resp === undefined) {
    const response = await fetch(url);
    if (!response.ok) {
      console.error("Error prefetching link");
      return;
    }
    await cache.put(url, response);
    resp = await cache.match(url);
  }
  if (resp === undefined) return;

  const newDocument = domParser.parseFromString(await resp.text(), "text/html");
  const newPageElem = newDocument.querySelector(pageSelector);
  if (!newPageElem) {
    console.error("Error finding the new page element when navigating!");
  } else {
    morphdom(page!, newPageElem, {
      onBeforeElUpdated: function (fromEl, toEl) {
        // spec - https://dom.spec.whatwg.org/#concept-node-equals
        if (fromEl.isEqualNode(toEl)) {
          return false;
        }

        return true;
      },
      childrenOnly: true,
    });

    morphdom(
      document.querySelector("head")!,
      newDocument.querySelector("head")!,
      {
        onBeforeElUpdated: function (fromEl, toEl) {
          // spec - https://dom.spec.whatwg.org/#concept-node-equals
          if (fromEl.isEqualNode(toEl)) {
            return false;
          }

          return true;
        },
        childrenOnly: true,
      }
    );

    window.history.pushState(undefined, "", url);
  }

  body?.classList.remove("mobile-sidebar-toggle");
  button?.removeAttribute("aria-pressed");

  hydratePage();
}

// Mount the handlers for links we are preloading and navigating with
function hydratePage() {
  if (button) button.addEventListener("click", toggleMobileNav);

  // When moving between docs and landing hide/show the mobile sidebar toggle
  const sidebarToggle = document.getElementById("sidebar-toggle");
  if (window.location.pathname === "/") {
    sidebarToggle?.classList.add("hidden");
  } else {
    sidebarToggle?.classList.remove("hidden");
  }

  const navLinks = document.querySelectorAll("a[rel=prefetch]");

  navLinks.forEach((link) => {
    // Preload page into the browser cache on hovering the navlink
    link.addEventListener("mouseover", async (e) => {
      if ((e.currentTarget as HTMLLinkElement)?.href === undefined) return;
      const url = new URL((e.currentTarget as HTMLLinkElement)?.href);

      // If the item is non in the browser cache
      if ((await cache.match(url)) === undefined) {
        // and is not being fetched currently
        if (!pagesBeingFetched.has(url.toString())) {
          let resolve: () => void;
          const promise: Promise<undefined> = new Promise((res) => {
            resolve = () => res(undefined);
          });

          pagesBeingFetched.set(url.toString(), promise);

          const response = await fetch(url);
          if (!response.ok) {
            console.error("Error prefetching link");
            return;
          }
          await cache.put(url, response);
          // @ts-expect-error
          resolve();
        }
      }
    });

    // Load the page on click
    link.addEventListener("click", (e) => {
      e.preventDefault();
      if (isNavigating) return;
      isNavigating = true;

      if ((e.currentTarget as HTMLLinkElement)?.href === undefined) return;
      const url = new URL((e.currentTarget as HTMLLinkElement)?.href);
      navigate(url).then(() => (isNavigating = false));
    });
  });
}

addEventListener("DOMContentLoaded", hydratePage);

window.addEventListener("popstate", (e) => {
  navigate(new URL(window.location.toString()), true);
});

export {};
