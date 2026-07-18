import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import type { Theme } from "@/lib/types";

interface ThemeContextType {
  theme: Theme;
  fontFamily: string;
  codeFontFamily: string;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
  setFontFamily: (font: string) => void;
  setCodeFontFamily: (font: string) => void;
}

const ThemeContext = createContext<ThemeContextType>({
  theme: "light",
  fontFamily: "system-ui",
  codeFontFamily: "JetBrains Mono",
  toggleTheme: () => {},
  setTheme: () => {},
  setFontFamily: () => {},
  setCodeFontFamily: () => {},
});

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = localStorage.getItem("theme");
    return (saved === "dark" ? "dark" : "light") as Theme;
  });

  const [fontFamily, setFontFamilyState] = useState<string>(() => {
    return localStorage.getItem("fontFamily") || "system-ui";
  });

  const [codeFontFamily, setCodeFontFamilyState] = useState<string>(() => {
    return localStorage.getItem("codeFontFamily") || "JetBrains Mono";
  });

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", theme === "dark");
    localStorage.setItem("theme", theme);
  }, [theme]);

  useEffect(() => {
    document.documentElement.style.setProperty("--reader-font", fontFamily);
    localStorage.setItem("fontFamily", fontFamily);
  }, [fontFamily]);

  useEffect(() => {
    document.documentElement.style.setProperty("--reader-code-font", codeFontFamily);
    localStorage.setItem("codeFontFamily", codeFontFamily);
  }, [codeFontFamily]);

  const toggleTheme = () => {
    setThemeState((prev) => (prev === "light" ? "dark" : "light"));
  };

  const setTheme = (t: Theme) => {
    setThemeState(t);
  };

  const setFontFamily = (font: string) => {
    setFontFamilyState(font);
  };

  const setCodeFontFamily = (font: string) => {
    setCodeFontFamilyState(font);
  };

  return (
    <ThemeContext.Provider value={{ theme, fontFamily, codeFontFamily, toggleTheme, setTheme, setFontFamily, setCodeFontFamily }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}