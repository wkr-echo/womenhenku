import { cn } from "@/lib/utils";
import type { InputHTMLAttributes } from "react";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  icon?: React.ReactNode;
}

export function Input({ className, icon, ...props }: InputProps) {
  return (
    <div className="relative">
      {icon && (
        <div className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-tertiary)]">
          {icon}
        </div>
      )}
      <input
        className={cn(
          "w-full h-9 rounded-lg border border-[var(--border-color)] bg-[var(--bg-secondary)] px-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-tertiary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent-color)] focus:border-transparent transition-colors",
          icon && "pl-9",
          className
        )}
        {...props}
      />
    </div>
  );
}