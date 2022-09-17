const tailwindMdBase = require("@geoffcodesthings/tailwind-md-base");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{astro,html,js,jsx,md,mdx,ts,tsx,css}"],
  theme: {
    extend: {},
    markdownBase: {
      code: {
        backgroundColor: "none",
      },
    },
  },
  plugins: [tailwindMdBase()],
};
