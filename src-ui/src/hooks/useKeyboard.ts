import { useEffect, useCallback } from "react";
import { useApp } from "@/contexts/AppContext";

export function useKeyboardShortcuts() {
  const { viewMode, setViewMode, selectedEntry, entries, selectEntry, selectFeed } = useApp();

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Don't intercept when typing in inputs
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") {
        return;
      }

      switch (e.key.toLowerCase()) {
        case "j": {
          e.preventDefault();
          if (viewMode === "list" && entries.length > 0) {
            const currentIdx = entries.findIndex((en) => en.id === selectedEntry?.id);
            const nextIdx = Math.min(currentIdx + 1, entries.length - 1);
            selectEntry(entries[nextIdx]);
          }
          break;
        }
        case "k": {
          e.preventDefault();
          if (viewMode === "list" && entries.length > 0) {
            const currentIdx = entries.findIndex((en) => en.id === selectedEntry?.id);
            const prevIdx = Math.max(currentIdx - 1, 0);
            selectEntry(entries[prevIdx]);
          }
          break;
        }
        case "escape": {
          e.preventDefault();
          setViewMode("list");
          break;
        }
      }
    },
    [viewMode, entries, selectedEntry, selectEntry, setViewMode]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}