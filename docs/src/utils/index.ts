import { config } from "../config";
import { parseMarkdown } from "./markdownParse";

const baseMarkdownDir = "../../markdown/";
const markdownFiles = import.meta.glob("../../markdown/**/*.md", {
  as: "raw",
  eager: true,
});

export async function getPageData() {
  const sidebar = new Map();

  for (const [path, rawFile] of Object.entries(await markdownFiles)) {
    const pathElems = path
      .replace(".md", "")
      .replace(baseMarkdownDir, "")
      .split("/");
    if (pathElems.length === 0 || pathElems.length > 2) {
      throw new Error("TODO");
    }
    const { render, metadata } = parseMarkdown(rawFile);
    const url = pathElems.filter((v) => (v === "index" ? "" : v)).join("/");
    sidebar.set(url, {
      url,
      title:
        metadata?.title ||
        toTitleCase(pathElems[1] ? pathElems[1] : pathElems[0]),
      categoryName:
        pathElems.length === 1 ? config.seo.title : toTitleCase(pathElems[0]),
      categorySlug: pathElems.length === 1 ? "" : pathElems[0],
      html: render,
      sortByIndex: metadata?.index ?? 100,
    });
  }

  return sidebar;
}

export async function getSidebarData() {
  let categories = new Map();

  for (const [, page] of await getPageData()) {
    categories.set(page.categorySlug, {
      name: page.categoryName,
      children: [...(categories.get(page.categorySlug)?.children || []), page],
    });
  }

  categories = new Map(
    [...(categories.entries() || [])]
      .map((a) => {
        a[1].children = a[1].children.sort(
          (a, b) => a.sortByIndex - b.sortByIndex
        );
        return a;
      })
      .sort(
        (a, b) => a[1].children[0].sortByIndex - b[1].children[0].sortByIndex
      )
  );

  return categories;
}

function toTitleCase(str: string) {
  return str
    .toLowerCase()
    .replace(/(?:^|[\s-/])\w/g, function (match) {
      return match.toUpperCase();
    })
    .replaceAll("-", " ");
}
