export function initSearch() {
    const filterInput = document.getElementById('filter-input') as HTMLInputElement | null;

    if (filterInput) {
        filterInput.addEventListener('input', (e) => {
            const query = (e.target as HTMLInputElement).value.toLowerCase();
            const filterableItems = document.querySelectorAll('[data-filter-text]');

            filterableItems.forEach(item => {
                const text = item.getAttribute('data-filter-text');
                const matches = text && text.includes(query);

                if (item instanceof HTMLElement) {
                    if (matches) {
                        item.style.display = '';
                    } else {
                        item.style.display = 'none';
                    }
                }
            });
        });
    }
}
