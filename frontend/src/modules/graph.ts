export function initGraph() {
    const treeView = document.querySelector('.tree-view');
    if (!treeView) return;

    const nodes = document.querySelectorAll('.tree-node') as NodeListOf<HTMLElement>;
    const typeFilters = document.querySelectorAll('.type-filter') as NodeListOf<HTMLInputElement>;
    const expandAllBtn = document.getElementById('expand-all');
    const collapseAllBtn = document.getElementById('collapse-all');

    const expandedNodes = new Set<string>();

    function updateVisibility() {
        const activeTypes = new Set(
            Array.from(typeFilters)
                .filter(f => f.checked)
                .map(f => f.value)
        );

        nodes.forEach(node => {
            const id = node.getAttribute('data-id') || '';
            const parentId = node.getAttribute('data-parent') || '';
            const type = node.getAttribute('data-type') || '';

            let visible = false;

            // Top-level nodes (no parent) are always visible if type matches
            if (!parentId) {
                visible = true;
            } else if (expandedNodes.has(parentId)) {
                // Children are visible if parent is expanded
                visible = true;
            }

            // Must also match type filter
            if (visible && !activeTypes.has(type)) {
                visible = false;
            }

            if (visible) {
                node.classList.remove('hidden');
            } else {
                node.classList.add('hidden');
            }
        });
    }

    // Toggle button click handlers
    nodes.forEach(node => {
        const toggleBtn = node.querySelector('.tree-toggle') as HTMLButtonElement | null;
        if (toggleBtn) {
            toggleBtn.addEventListener('click', (e) => {
                debugger;
                e.preventDefault();
                e.stopPropagation();

                const id = node.getAttribute('data-id');
                if (!id) return;

                if (expandedNodes.has(id)) {
                    expandedNodes.delete(id);
                    toggleBtn.classList.remove('expanded');
                    const icon = toggleBtn.querySelector('.toggle-icon');
                    if (icon) icon.textContent = '+';
                } else {
                    expandedNodes.add(id);
                    toggleBtn.classList.add('expanded');
                    const icon = toggleBtn.querySelector('.toggle-icon');
                    if (icon) icon.textContent = '−';
                }

                updateVisibility();
            });
        }
    });

    // Type filter change handlers
    typeFilters.forEach(filter => {
        filter.addEventListener('change', updateVisibility);
    });

    // Expand all button
    if (expandAllBtn) {
        expandAllBtn.addEventListener('click', () => {
            nodes.forEach(node => {
                const hasChildren = node.getAttribute('data-has-children') === 'true';
                if (hasChildren) {
                    const id = node.getAttribute('data-id');
                    if (id) {
                        expandedNodes.add(id);
                        const toggleBtn = node.querySelector('.tree-toggle');
                        if (toggleBtn) {
                            toggleBtn.classList.add('expanded');
                            const icon = toggleBtn.querySelector('.toggle-icon');
                            if (icon) icon.textContent = '−';
                        }
                    }
                }
            });
            updateVisibility();
        });
    }

    // Collapse all button
    if (collapseAllBtn) {
        collapseAllBtn.addEventListener('click', () => {
            expandedNodes.clear();
            nodes.forEach(node => {
                const toggleBtn = node.querySelector('.tree-toggle');
                if (toggleBtn) {
                    toggleBtn.classList.remove('expanded');
                    const icon = toggleBtn.querySelector('.toggle-icon');
                    if (icon) icon.textContent = '+';
                }
            });
            updateVisibility();
        });
    }

    // Initialize visibility
    updateVisibility();
}
