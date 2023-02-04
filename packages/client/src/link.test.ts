import { describe, it } from "vitest";
import { HasAnyLinkFlags, HasLinkFlags } from "./link";
import { assertTy } from "./utils.test";

describe("Links", () => {
  it("HasLinkFlags & HasAnyLinkFlags", async () => {
    assertTy<HasLinkFlags<{ terminatedLink: true }, "terminatedLink">, true>();
    assertTy<
      HasLinkFlags<{ built: true }, "terminatedLink" | "built">,
      false
    >();
    assertTy<
      HasLinkFlags<
        { terminatedLink: true; built: true },
        "terminatedLink" | "built"
      >,
      true
    >();
    assertTy<HasLinkFlags<{}, "terminatedLink" | "built">, false>();

    assertTy<
      HasAnyLinkFlags<{ terminatedLink: true }, "terminatedLink">,
      true
    >();
    assertTy<
      HasAnyLinkFlags<
        { terminatedLink: true; built: true },
        "terminatedLink" | "built"
      >,
      true
    >();
    assertTy<HasAnyLinkFlags<{}, "terminatedLink" | "built">, false>();
  });
});
