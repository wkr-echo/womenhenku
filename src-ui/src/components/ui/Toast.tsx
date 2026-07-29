import { useEffect, useState, useCallback } from "react";

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
    }, 6000);
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
          style={{
            padding: "12px 16px",
            borderRadius: 8,
            boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
            fontSize: 14,
            fontWeight: 500,
            maxWidth: 320,
            color: "#fff",
            backgroundColor:
              t.type === "success" ? "#16a34a" :
              t.type === "error" ? "#dc2626" :
              "var(--bg-tertiary)",
          }}
        >
          {t.message}
        </div>
      ))}
    </div>
  );
}