import { config } from "../config";

const baseMarkdownDir = "../../markdown/";
const markdownFiles = import.meta.glob("../../markdown/**/*.md");

export interface Page {
  url: string;
  title: string;
  header?: string;
  categoryName: string;
  categorySlug: string;
  html: string;
  headers: {
    title: string;
    slug: string;
  }[];
  sortByIndex: number;
}

export async function getPageData(): Promise<Map<string, Page>> {
  const sidebar = new Map<string, Page>();

  // TODO
  for (const [path, fileFn] of Object.entries(await markdownFiles)) {
    const pathElems = path
      .replace(".md", "")
      .replace(baseMarkdownDir, "")
      .split("/");
    if (pathElems.length === 0 || pathElems.length > 2) {
      throw new Error("TODO: You broke it stupid.");
    }

    const file = (await fileFn()) as any;
    const url = pathElems.filter((v) => (v === "index" ? "" : v)).join("/");
    sidebar.set(url, {
      url,
      title:
        file.frontmatter?.title ||
        toTitleCase((pathElems[1] ? pathElems[1] : pathElems[0])!),
      header: file.frontmatter?.header,
      categoryName:
        pathElems.length === 1 ? config.seo.title : toTitleCase(pathElems[0]!),
      categorySlug: pathElems.length === 1 ? "" : pathElems[0]!,
      html: file.compiledContent(),
      headers: file
        .getHeadings()
        .filter(({ depth }: any) => depth > 0 && depth < 4)
        .map((header: any) => ({
          title: header.text,
          slug: header.slug,
        })),
      sortByIndex: file.frontmatter?.index ?? 100,
    });
  }

  return sidebar;
}
export interface Category {
  name: string;
  children: Page[];
}

export async function getSidebarData(): Promise<Map<string, Category>> {
  let categories = new Map<string, Category>();

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
        (a, b) =>
          a[1]!.children[0]!.sortByIndex - b[1]!.children[0]!.sortByIndex
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
