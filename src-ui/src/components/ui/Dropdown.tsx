import { cn } from "@/lib/utils";
import { useState, useRef, useEffect } from "react";

interface DropdownItem {
  label: string;
  value: string;
  icon?: React.ReactNode;
}

interface DropdownProps {
  items: DropdownItem[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}

export function Dropdown({ items, value, onChange, placeholder = "请选择", className }: DropdownProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const selected = items.find((i) => i.value === value);

  return (
    <div ref={ref} className={cn("relative", className)}>
      <button
        onClick={() => setOpen(!open)}
        className="w-full h-9 rounded-lg border border-[var(--border-color)] bg-[var(--bg-secondary)] px-3 text-sm text-left flex items-center justify-between hover:border-[var(--text-tertiary)] transition-colors"
      >
        <span className={selected ? "text-[var(--text-primary)]" : "text-[var(--text-tertiary)]"}>
          {selected?.label ?? placeholder}
        </span>
        <svg
          className={cn("w-4 h-4 text-[var(--text-tertiary)] transition-transform", open && "rotate-180")}
          fill="none" stroke="currentColor" viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      {open && (
        <div className="absolute z-20 mt-1 w-full rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] shadow-lg py-1 animate-fade-in">
          {items.map((item) => (
            <button
              key={item.value}
              onClick={() => {
                onChange(item.value);
                setOpen(false);
              }}
              className={cn(
                "w-full text-left px-3 py-2 text-sm flex items-center gap-2 hover:bg-[var(--bg-tertiary)] transition-colors",
                item.value === value && "bg-[var(--bg-tertiary)] text-[var(--accent-color)]"
              )}
            >
              {item.icon}
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}