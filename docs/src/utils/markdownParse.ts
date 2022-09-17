import parseMarkdownMetadata from "markdown-yaml-metadata-parser";
import { marked } from "marked";
import { writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import { exec } from "node:child_process";

import prism from "prismjs";
import "prismjs/components/prism-rust";
import "prismjs/components/prism-typescript";

marked.setOptions({
  highlight: (code, lang) => {
    if (lang === "rust-test") {
      // rustRustUnitTest(
      //   code,
      //   [
      //     `[package]`,
      //     `name = "${createHash("md5").update(code).digest("hex")}"`,
      //     `version = "0.1.0"`,
      //     "",
      //   ]
      //     .concat(
      //       code
      //         .split("\n")
      //         .filter((l) => l.startsWith(`//!`))
      //         .map((v) => v.slice(4))
      //     )
      //     .join("\n")
      // );
      // code = code
      //   .split("\n")
      //   .filter(
      //     (l) =>
      //       !l.startsWith(`//!`) && !l.endsWith(" // hidden") && l !== "#[test]"
      //   );
      // if (code[0] === "") code = code.splice(1);
      // code = code.join("\n");
      lang = "rust";
    }

    if (prism.languages[lang]) {
      return prism.highlight(code, prism.languages[lang], lang);
    } else {
      return code;
    }
  },
});

const rustTestsPath = "./_test/";

function rustRustUnitTest(code: string, cargoToml: string) {
  const codeHash = createHash("md5").update(code).digest("hex");
  mkdirSync(`${rustTestsPath}/${codeHash}/src/`, { recursive: true });
  writeFileSync(`${rustTestsPath}/${codeHash}/src/lib.rs`, code);
  writeFileSync(`${rustTestsPath}/${codeHash}/Cargo.toml`, cargoToml);

  // TODO
  // exec("cargo test", (error, stdout, stderr) => {
  //   if (error) {
  //     console.log(`error: ${error.message}`);
  //     return;
  //   }
  //   if (stderr) {
  //     console.log(`stderr: ${stderr}`);
  //     return;
  //   }
  //   console.log(`stdout: ${stdout}`);
  // });
}

export interface MarkdownPageData {
  name?: string;
  index?: number;
  new?: boolean;
}

interface MarkdownParsed {
  render: string;
  metadata?: MarkdownPageData;
}

export function parseMarkdown(markdownRaw: string): MarkdownParsed {
  let metadata: MarkdownPageData | undefined = undefined;
  let withoutMetadata = markdownRaw;
  try {
    const parsed = parseMarkdownMetadata(markdownRaw);
    metadata = parsed.metadata;
    withoutMetadata = parsed.content;
  } catch (e) {
    // console.warn('failed to parse markdown', e);
    // this doesn't matter
  }
  let markdownAsHtml = marked(withoutMetadata);

  // make all non local links open in new tab
  markdownAsHtml = markdownAsHtml.replaceAll(
    '<a href="http',
    `<a target="_blank" rel="noreferrer" href="http`
  );

  // console.log(markdownAsHtml);
  // console.log(Prism.highlightElement(markdownAsHtml));

  // const rawSplit = markdownRaw.split(":::");

  // custom support for "slots" like vuepress
  // markdownAsHtml = markdownAsHtml
  //   .split(":::")
  //   .map((text, index) => {
  //     if (index % 2 === 0) {
  //       return text;
  //     } else {
  //       const rawText = rawSplit[index],
  //         meta = rawText.split(/\r?\n/)[0].trim(),
  //         kind = meta.split(" ")[0],
  //         name = meta.split(" ")[1],
  //         extra = meta.substring(kind.length + name.length + 2),
  //         content = text.substring(meta.length + 1, text.length).trim();

  //       // console.log({ kind, name, extra, content });

  //       switch (kind) {
  //         case "slot":
  //           return `<div class="slot-block ${name}"><h5 class="slot-block-title">${
  //             extra || name
  //           }</h5><p class="slot-block-content">${content}</p></div>`;
  //           break;

  //         default:
  //           break;
  //       }
  //     }
  //   })
  //   .join("");

  return {
    render: markdownAsHtml,
    metadata,
  };
}
