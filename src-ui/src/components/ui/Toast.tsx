import { useEffect, useState, useCallback } from "react";
import { cn } from "@/lib/utils";

interface ToastItem {
  id: number;
  message: string;
  type: "success" | "error" | "info";
}

let toastId = 0;
let addToastFn: ((msg: string, type: ToastItem["type"]) => void) | null = null;

export function toast(message: string, type: ToastItem["type"] = "info") {
  addToastFn?.(message, type);
}

export function ToastContainer() {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const addToast = useCallback((message: string, type: ToastItem["type"]) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 3000);
  }, []);

  useEffect(() => {
    addToastFn = addToast;
    return () => {
      addToastFn = null;
    };
  }, [addToast]);

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
      {toasts.map((t) => (
        <div
          key={t.id}
          className={cn(
            "px-4 py-3 rounded-lg shadow-lg text-sm font-medium animate-fade-in max-w-sm",
            {
              "bg-green-600 text-white": t.type === "success",
              "bg-red-600 text-white": t.type === "error",
              "bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)]":
                t.type === "info",
            }
          )}
        >
          {t.message}
        </div>
      ))}
    </div>
  );
}