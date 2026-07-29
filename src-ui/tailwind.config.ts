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
        ui: {
          primary: {
            50: "#eef2f8",
            100: "#d5dff0",
            200: "#b0c4e0",
            300: "#8aa5cb",
            400: "#6888b5",
            500: "#4b6a9e",
            600: "#3a5484",
            700: "#2e4268",
            800: "#1e2d4a",
            900: "#0f1a2e",
          },
          success: {
            50: "#edf8f0",
            100: "#d0efda",
            200: "#a6dfbc",
            300: "#70c696",
            400: "#4aad78",
            500: "#2e9360",
            600: "#23784d",
            700: "#1a5e3c",
            800: "#12472c",
            900: "#0a2e1c",
          },
          warning: {
            50: "#fef7ea",
            100: "#fde9c4",
            200: "#f9d48d",
            300: "#f5b85a",
            400: "#f0a034",
            500: "#e88a1a",
            600: "#cc6e10",
            700: "#a8560c",
            800: "#85410a",
            900: "#6b3408",
          },
          error: {
            50: "#fef0f0",
            100: "#fdd5d5",
            200: "#fbaaaa",
            300: "#f57575",
            400: "#ed4e4e",
            500: "#de2e2e",
            600: "#c42020",
            700: "#a01818",
            800: "#821212",
            900: "#6a0e0e",
          },
          info: {
            50: "#eef5fc",
            100: "#d2e6f7",
            200: "#a5cbef",
            300: "#72abe3",
            400: "#4a8fd8",
            500: "#2770c9",
            600: "#1b58a5",
            700: "#164382",
            800: "#113161",
            900: "#0b2040",
          },
          neutral: {
            50: "#f5f6f8",
            100: "#e6e9ee",
            200: "#ced3dc",
            300: "#a8b0be",
            400: "#838b9d",
            500: "#636c80",
            600: "#4e5566",
            700: "#3d434f",
            800: "#2b3039",
            900: "#181b22",
          },
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
        display: ["DM Serif Display", "Georgia", "serif"],
        heading: ["DM Serif Display", "Georgia", "serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      },
      borderRadius: {
        "ui-sm": "4px",
        "ui-md": "8px",
        "ui-lg": "12px",
        "ui-xl": "16px",
      },
      boxShadow: {
        "ui-1": "0 1px 2px rgba(24,27,34,.06), 0 1px 1px rgba(24,27,34,.04)",
        "ui-2": "0 4px 8px -2px rgba(24,27,34,.10)",
        "ui-3": "0 8px 24px -8px rgba(24,27,34,.18)",
        "ui-4": "0 16px 40px -12px rgba(24,27,34,.24)",
        "ui-5": "0 24px 60px -20px rgba(24,27,34,.30)",
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