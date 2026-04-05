/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        mono: ["JetBrains Mono", "Menlo", "Monaco", "monospace"],
        sans: ["Inter", "system-ui", "sans-serif"],
      },
      colors: {
        canvas: {
          50: "#FAFAF9",
          100: "#F5F5F4",
          200: "#E5E5E3",
        },
        primary: {
          500: "#4A83DD",
          600: "#3D6DC4",
        },
      },
    },
  },
  plugins: [],
};
