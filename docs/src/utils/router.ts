// This is a mini single page style router which is used on the frontend.

import morphdom from "morphdom";

const links = document.querySelectorAll(".route");
const domParser = new DOMParser();

const pages = new Map();

function doNavigate(href: string, body: string) {
  const newDocument = domParser.parseFromString(body, "text/html");
  morphdom(document.head, newDocument.head);
  morphdom(document.body, newDocument.body);
  window.history.pushState(undefined, "", href);
}

links.forEach((link) => {
  // Preload page on hovering the navlink
  link.addEventListener("mouseover", (e) => {
    if (e.target.href === undefined) return;
    if (!pages.get(e.target.href)) {
      fetch(e.target.href)
        .then((res) => res.text())
        .then((body) => {
          pages.set(e.target.href, body);
        });
    }
  });

  // Load the page on click
  link.addEventListener("click", (e) => {
    if (e.target.href === undefined) return;
    e.preventDefault();

    const cachedPage = pages.get(e.target.href);
    if (cachedPage == undefined) {
      fetch(e.target.href)
        .then((res) => res.text())
        .then((body) => {
          pages.set(e.target.href, body);
          doNavigate(e.target.href, body);
        });
    } else {
      doNavigate(e.target.href, cachedPage);
    }
  });
});

window.addEventListener("popstate", (e) => {
  const cachedPage = pages.get(window.location.pathname);
  if (cachedPage == undefined) {
    fetch(window.location.pathname)
      .then((res) => res.text())
      .then((body) => {
        pages.set(window.location.pathname, body);
        doNavigate(window.location.pathname, body);
      });
  } else {
    doNavigate(window.location.pathname, cachedPage);
  }
});
