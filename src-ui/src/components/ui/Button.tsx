import { cn } from "@/lib/utils";
import type { ButtonHTMLAttributes, ReactNode } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: "sm" | "md" | "lg";
  children: ReactNode;
}

export function Button({
  variant = "primary",
  size = "md",
  className,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent-color)] disabled:opacity-50 disabled:pointer-events-none",
        {
          "bg-[var(--accent-color)] text-white hover:opacity-90": variant === "primary",
          "bg-[var(--bg-tertiary)] text-[var(--text-primary)] hover:bg-[var(--border-color)]":
            variant === "secondary",
          "text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]":
            variant === "ghost",
          "bg-[var(--danger-color)] text-white hover:opacity-90": variant === "danger",
        },
        {
          "h-8 px-3 text-xs": size === "sm",
          "h-9 px-4 text-sm": size === "md",
          "h-10 px-5 text-base": size === "lg",
        },
        className
      )}
      {...props}
    >
      {children}
    </button>
  );
}