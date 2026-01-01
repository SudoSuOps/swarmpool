/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  theme: {
    extend: {
      colors: {
        // SwarmHive Medical Palette
        'swarm': {
          // Primary - Medical Blue
          'blue': '#0066CC',
          'blue-light': '#3399FF',
          'blue-dark': '#004499',
          
          // Accent - Honey/Amber (subtle nod to bees)
          'honey': '#F5A623',
          'honey-light': '#FFD93D',
          'honey-dark': '#CC8800',
          
          // Backgrounds
          'bg-dark': '#0A1628',
          'bg-card': '#0F1D32',
          'bg-elevated': '#152238',
          
          // Borders
          'border': '#1E3A5F',
          'border-light': '#2A4A6F',
          
          // Text
          'text': '#E8F0F8',
          'text-dim': '#8BA3C0',
          'text-muted': '#5A7A9A',
          
          // Status
          'success': '#00D68F',
          'warning': '#FFAA00',
          'error': '#FF3D71',
          'online': '#00E096',
        }
      },
      fontFamily: {
        'sans': ['Inter', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      boxShadow: {
        'glow': '0 0 20px rgba(0, 102, 204, 0.15)',
        'glow-honey': '0 0 20px rgba(245, 166, 35, 0.15)',
        'card': '0 4px 24px rgba(0, 0, 0, 0.25)',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'float': 'float 6s ease-in-out infinite',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        }
      }
    },
  },
  plugins: [],
}
