import type { Config } from "tailwindcss";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const tailwindcssAnimate = require("tailwindcss-animate");

export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        sidebar: {
          bg: "var(--sidebar-bg)",
          hover: "var(--sidebar-hover)",
          active: "var(--sidebar-active)",
        },
      },
      fontFamily: {
        sans: [
          "Inter",
          "PingFang SC",
          "Microsoft YaHei",
          "Hiragino Sans GB",
          "WenQuanYi Micro Hei",
          "system-ui",
          "-apple-system",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "Fira Code",
          "monospace",
        ],
      },
      animation: {
        "slide-in": "slideIn 0.2s ease-out",
        "fade-in": "fadeIn 0.15s ease-out",
        "spin-slow": "spin 2s linear infinite",
      },
      keyframes: {
        slideIn: {
          from: { transform: "translateX(-100%)", opacity: "0" },
          to: { transform: "translateX(0)", opacity: "1" },
        },
        fadeIn: {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
      },
    },
  },
  plugins: [tailwindcssAnimate],
} satisfies Config;