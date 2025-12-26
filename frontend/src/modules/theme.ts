// Theme switching - run immediately at module load to prevent flash
(function() {
    const stored = localStorage.getItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const theme = stored || (prefersDark ? 'dark' : 'light');
    document.documentElement.setAttribute('data-theme', theme);
})();

export function initTheme() {
    // Set up theme toggle button (requires DOM to be ready)
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        const updateIcon = () => {
            const current = document.documentElement.getAttribute('data-theme');
            themeToggle.textContent = current === 'dark' ? '☀️' : '🌙';
        };

        updateIcon();

        themeToggle.addEventListener('click', () => {
            const current = document.documentElement.getAttribute('data-theme');
            const next = current === 'dark' ? 'light' : 'dark';
            document.documentElement.setAttribute('data-theme', next);
            localStorage.setItem('theme', next);
            updateIcon();
        });
    }
}