export function initListFeatures() {
    // Epic children toggle
    const toggleButtons = document.querySelectorAll('.toggle-children');
    toggleButtons.forEach(button => {
        button.addEventListener('click', () => {
            const epicItem = button.closest('.epic-item');
            if (!epicItem) return;
            const children = epicItem.querySelector('.epic-children') as HTMLElement | null;

            if (children) {
                const isCollapsed = children.classList.contains('collapsed');
                children.classList.toggle('collapsed');
                button.classList.toggle('expanded');

                if (isCollapsed) {
                    children.style.maxHeight = children.scrollHeight + 'px';
                    children.style.opacity = '1';
                } else {
                    children.style.maxHeight = '0';
                    children.style.opacity = '0';
                }
            }
        });
    });
}
