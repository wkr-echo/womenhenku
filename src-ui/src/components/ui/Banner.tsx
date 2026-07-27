import { cn } from "@/lib/utils";

type BannerLevel = "error" | "warning" | "info";

interface BannerProps {
  message: string;
  level?: BannerLevel;
  onDismiss?: () => void;
  className?: string;
}

export function Banner({ message, level = "error", onDismiss, className }: BannerProps) {
  const colors = {
    error: "bg-red-50 border-red-300 text-red-700",
    warning: "bg-yellow-50 border-yellow-300 text-yellow-700",
    info: "bg-blue-50 border-blue-300 text-blue-700",
  };

  return (
    <div className={cn("px-4 py-2.5 text-sm border-b flex items-center justify-between", colors[level], className)}>
      <span>{message}</span>
      {onDismiss && (
        <button onClick={onDismiss} className="ml-2 text-current opacity-60 hover:opacity-100">
          ✕
        </button>
      )}
    </div>
  );
}
