import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import type { Theme } from "@/lib/types";

interface ThemeContextType {
  theme: Theme;
  fontFamily: string;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
  setFontFamily: (font: string) => void;
}

const ThemeContext = createContext<ThemeContextType>({
  theme: "light",
  fontFamily: "system-ui",
  toggleTheme: () => {},
  setTheme: () => {},
  setFontFamily: () => {},
});

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = localStorage.getItem("theme");
    return (saved === "dark" ? "dark" : "light") as Theme;
  });

  const [fontFamily, setFontFamilyState] = useState<string>(() => {
    return localStorage.getItem("fontFamily") || "system-ui";
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

  const toggleTheme = () => {
    setThemeState((prev) => (prev === "light" ? "dark" : "light"));
  };

  const setTheme = (t: Theme) => {
    setThemeState(t);
  };

  const setFontFamily = (font: string) => {
    setFontFamilyState(font);
  };

  return (
    <ThemeContext.Provider value={{ theme, fontFamily, toggleTheme, setTheme, setFontFamily }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  return useContext(ThemeContext);
}