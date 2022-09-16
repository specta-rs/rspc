const root = document.documentElement;
const theme = localStorage.getItem("theme");
if (
  theme === "dark" ||
  (!theme && window.matchMedia("(prefers-color-scheme: dark)").matches)
) {
  root.classList.add("theme-dark");
} else {
  root.classList.remove("theme-dark");
}
