import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        ink: {
          950: '#0b0c0f',
          900: '#13151a',
          800: '#1c1f26',
          700: '#272b34',
          600: '#383d49',
        },
        accent: {
          DEFAULT: '#7c6cf6',
          dim: '#5b4fc4',
        },
      },
      fontFamily: {
        sans: ['"Inter"', '"Segoe UI"', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', '"SFMono-Regular"', 'monospace'],
      },
    },
  },
  plugins: [],
};

export default config;
