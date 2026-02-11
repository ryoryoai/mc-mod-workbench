import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        border: "hsl(217 33% 24%)",
        input: "hsl(217 33% 24%)",
        ring: "hsl(212 100% 48%)",
        background: "hsl(222 47% 11%)",
        foreground: "hsl(210 40% 98%)",
      },
    },
  },
  plugins: [],
};

export default config;
