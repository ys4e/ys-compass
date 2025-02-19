import { Config } from "tailwindcss";
import tailwindcssAnimate from "tailwindcss-animate";

export default {
    content: ["./src/**/*.{html,js,tsx,ts}"],
    mode: "jit",
    theme: {
        extend: {
            colors: {
                black: {
                    100: "rgba(0, 0, 10, 0.4)",
                    200: "rgba(0, 0, 10, 0.6)",
                    900: "#000000"
                },
                white: {
                    0: "rgba(255, 255, 255, 0)",
                    7: "rgba(255, 255, 255, 0.07)",
                    100: "#ffffff"
                },
                aqua: "#4360A2",
                "light-blue": "rgba(100, 130, 255, 0.2)",
                "dark-blue": "#204e8a",

                // Below are shadcn. Tailwind colors.
                border: "hsl(var(--border))",
                input: "hsl(var(--input))",
                ring: "hsl(var(--ring))",
                background: "hsl(var(--background))",
                foreground: "hsl(var(--foreground))",
                primary: {
                    DEFAULT: "hsl(var(--primary))",
                    foreground: "hsl(var(--primary-foreground))",
                },
                secondary: {
                    DEFAULT: "hsl(var(--secondary))",
                    foreground: "hsl(var(--secondary-foreground))",
                },
                destructive: {
                    DEFAULT: "hsl(var(--destructive))",
                    foreground: "hsl(var(--destructive-foreground))",
                },
                muted: {
                    DEFAULT: "hsl(var(--muted))",
                    foreground: "hsl(var(--muted-foreground))",
                },
                accent: {
                    DEFAULT: "hsl(var(--accent))",
                    foreground: "hsl(var(--accent-foreground))",
                },
                popover: {
                    DEFAULT: "hsl(var(--popover))",
                    foreground: "hsl(var(--popover-foreground))",
                },
                card: {
                    DEFAULT: "hsl(var(--card))",
                    foreground: "hsl(var(--card-foreground))",
                },
                sidebar: {
                    DEFAULT: "hsl(var(--sidebar-background))",
                    foreground: "hsl(var(--sidebar-foreground))",
                    primary: "hsl(var(--sidebar-primary))",
                    "primary-foreground": "hsl(var(--sidebar-primary-foreground))",
                    accent: "hsl(var(--sidebar-accent))",
                    "accent-foreground": "hsl(var(--sidebar-accent-foreground))",
                    border: "hsl(var(--sidebar-border))",
                    ring: "hsl(var(--sidebar-ring))",
                }
            },
            borderRadius: {
                lg: "var(--radius)",
                md: "calc(var(--radius) - 2px)",
                sm: "calc(var(--radius) - 4px)",
            },
            backdropSaturate: {
                90: "90%",
                120: "120%"
            },
            backdropBrightness: {
                80: "80%"
            }
        }
    },
    darkMode: ["class", "[data-mode=\"dark\"]"],
    plugins: [
        tailwindcssAnimate
    ]
} satisfies Config;
