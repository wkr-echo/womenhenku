import { useEffect, useCallback } from "react";
import { useApp } from "@/contexts/AppContext";

export function useKeyboardShortcuts() {
  const { viewMode, setViewMode, selectedEntry, entries, selectEntry } = useApp();

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Don't intercept when typing in inputs or textareas
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") {
        return;
      }

      // Only handle j/k navigation when not in reader view (reader handles s/t/n itself)
      switch (e.key.toLowerCase()) {
        case "j": {
          e.preventDefault();
          if (viewMode === "list" && entries.length > 0) {
            const currentIdx = entries.findIndex((en) => en.id === selectedEntry?.id);
            const nextIdx = currentIdx < 0 ? 0 : Math.min(currentIdx + 1, entries.length - 1);
            if (nextIdx >= 0) selectEntry(entries[nextIdx]);
          }
          break;
        }
        case "k": {
          e.preventDefault();
          if (viewMode === "list" && entries.length > 0) {
            const currentIdx = entries.findIndex((en) => en.id === selectedEntry?.id);
            const prevIdx = currentIdx < 0 ? 0 : Math.max(currentIdx - 1, 0);
            if (prevIdx >= 0) selectEntry(entries[prevIdx]);
          }
          break;
        }
        case "escape": {
          // Only handle escape in list mode (reader handles its own escape for panels)
          break;
        }
      }
    },
    [viewMode, entries, selectedEntry, selectEntry]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}