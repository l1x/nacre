document.addEventListener('DOMContentLoaded', () => {
    const filterInput = document.getElementById('filter-input');
    const issueItems = document.querySelectorAll('.issue-item');

    if (filterInput) {
        filterInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase();
            
            issueItems.forEach(item => {
                const text = item.getAttribute('data-filter-text');
                if (text && text.includes(query)) {
                    item.style.display = 'flex';
                } else {
                    item.style.display = 'none';
                }
            });
        });
    }
});