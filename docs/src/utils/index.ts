const baseMarkdownDir = "../../markdown/";
const markdownFiles = import.meta.glob("../../markdown/**/*.md");

export async function getPageData() {
  const sidebar = new Map();

  for (const [path, component] of Object.entries(await markdownFiles)) {
    const pathElems = path
      .replace(".md", "")
      .replace(baseMarkdownDir, "")
      .split("/");

    if (pathElems.length === 0 || pathElems.length > 2) {
      throw new Error("TOD");
    }

    const url = pathElems
      .filter((v) => (v === "index" ? undefined : v))
      .join("/");
    sidebar.set(url, {
      url,
      title: pathElems[1]
        ? toTitleCase(pathElems[1])
        : toTitleCase(pathElems[0]),
      categoryName: toTitleCase(pathElems[0]),
      categorySlug: pathElems[0],
      component,
    });
  }

  return sidebar;
}

export async function getSidebarData() {
  const categories = new Map();

  for (const [, page] of await getPageData()) {
    categories.set(page.categorySlug, {
      name: page.categoryName,
      children: [...(categories.get(page.categorySlug)?.children || []), page],
    });
  }

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
