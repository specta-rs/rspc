import Sidebar from "../components/Sidebar";
import { config } from "../config";

const themeScript = await (
  await import.meta.glob("../utils/*", { as: "raw" })["../utils/theme.js"]
)();

export default function Page(props: { activePath: string; children: any }) {
  return (
    <html lang="en" dir="ltr">
      <head>
        <meta charset="UTF-8" />
        <meta http-equiv="X-UA-Compatible" content="IE=edge" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        {/* TODO: Join page title into SEO title */}
        <title>{config.seo.title}</title>
        {config.seo?.description && (
          <meta name="description" content={config.seo.description} />
        )}
        {config.seo?.keywords && (
          <meta name="keywords" content={config.seo.keywords.join(", ")} />
        )}
        <link rel="icon" type="image/x-icon" href="/favicon.ico" />
        {/* TODO: <link rel="sitemap" href="/sitemap-index.xml" /> */}
        {/* <link rel="canonical" href={new URL(Astro.url.pathname, Astro.site)} /> */}
        {/* OpenGraph Tags */}
        {/* <meta property="og:title" content={content.title !== "rspc" ? `${content.title} | ${CONFIG.SITE.title}` : CONFIG.SITE.title} />
        <meta property="og:type" content="article" />
        <meta property="og:url" content={canonicalURL} />
        <meta property="og:locale" content={content.ogLocale ?? SITE.defaultLanguage} />
        <meta property="og:image" content={canonicalImageSrc} />
        <meta property="og:image:alt" content={imageAlt} />
        <meta name="description" property="og:description" content={content.description ? content.description : SITE.description} />
        <meta property="og:site_name" content={SITE.title} /> */}
        {/* Twitter Tags */}
        {/* <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:site" content={OPEN_GRAPH.twitter} />
        <meta name="twitter:title" content={formattedContentTitle} />
        <meta name="twitter:description" content={content.description ? content.description : SITE.description} />
        <meta name="twitter:image" content={canonicalImageSrc} />
        <meta name="twitter:image:alt" content={imageAlt} /> */}
        {config.seo?.customHead || []}
        <script innerHTML={themeScript}></script>
      </head>
      <body class="h-screen flex text-black dark:text-white dark:bg-[#242424]">
        <Sidebar activePath={props.activePath} />
        <main class="h-full overflow-none">{props.children}</main>
      </body>
    </html>
  );
}
