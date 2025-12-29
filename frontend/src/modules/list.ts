export function initListFeatures() {
    // Event delegation: handle toggle-children clicks at document level
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        const button = target.closest('.toggle-children');
        if (!button) return;

        const epicItem = button.closest('.epic-item');
        if (!epicItem) return;

        const children = epicItem.querySelector('.epic-children') as HTMLElement | null;
        if (!children) return;

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
    });
}
