// Theme switching - run immediately at module load to prevent flash
const LIGHT_THEMES = ['nacre-light', 'catppuccin-latte'];

(function() {
    const stored = localStorage.getItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const theme = stored || (prefersDark ? 'nacre-dark' : 'nacre-light');
    document.documentElement.setAttribute('data-theme', theme);
    // Set syntax theme based on light/dark
    const syntaxTheme = LIGHT_THEMES.includes(theme) ? 'light' : 'dark';
    document.documentElement.setAttribute('data-syntax', syntaxTheme);
})();

export function initTheme() {
    // Set up theme select dropdown (requires DOM to be ready)
    const themeSelect = document.getElementById('theme-select') as HTMLSelectElement | null;
    if (themeSelect) {
        // Set initial value
        const current = document.documentElement.getAttribute('data-theme') || 'nacre-dark';
        themeSelect.value = current;

        themeSelect.addEventListener('change', () => {
            const newTheme = themeSelect.value;
            document.documentElement.setAttribute('data-theme', newTheme);
            localStorage.setItem('theme', newTheme);
            // Update syntax theme based on light/dark
            const syntaxTheme = LIGHT_THEMES.includes(newTheme) ? 'light' : 'dark';
            document.documentElement.setAttribute('data-syntax', syntaxTheme);
        });
    }
}
