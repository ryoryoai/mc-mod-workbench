import * as React from "react";
import { cn } from "@/lib/utils";

export function Textarea(
  props: React.TextareaHTMLAttributes<HTMLTextAreaElement>,
) {
  const { className, ...rest } = props;
  return (
    <textarea
      className={cn(
        "w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none placeholder:text-slate-400 focus-visible:ring-2 focus-visible:ring-blue-500",
        className,
      )}
      {...rest}
    />
  );
}
