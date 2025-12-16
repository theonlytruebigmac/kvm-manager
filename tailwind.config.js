/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
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
        // Desktop-native color tokens
        window: {
          bg: "var(--window-bg)",
          border: "var(--window-border)",
        },
        panel: {
          bg: "var(--panel-bg)",
          border: "var(--panel-border)",
        },
        toolbar: {
          bg: "var(--toolbar-bg)",
          border: "var(--toolbar-border)",
          hover: "var(--toolbar-hover)",
        },
        sidebar: {
          bg: "var(--sidebar-bg)",
          border: "var(--sidebar-border)",
          hover: "var(--sidebar-hover)",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      spacing: {
        // Desktop-appropriate spacing
        'toolbar': '0.5rem',  // 8px
        'panel': '1rem',      // 16px
        'tight': '0.5rem',    // 8px
      },
      fontSize: {
        // Desktop font sizes
        'desktop-xs': '11px',
        'desktop-sm': '12px',
        'desktop-base': '13px',
        'desktop-lg': '14px',
      },
    },
  },
  plugins: [],
}
