import { render } from "@testing-library/svelte";
import { test } from "vitest";

import HooksTest from "./HooksTest.svelte";
import UtilsTest from "./UtilsTest.svelte";

test("hooks", () => {
	render(HooksTest);
});

test("utils", () => {
	render(UtilsTest);
});
