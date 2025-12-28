export function initGraph() {
    const treeView = document.querySelector('.tree-view');
    if (!treeView) return;

    const nodes = document.querySelectorAll('.tree-node') as NodeListOf<HTMLElement>;
    const typeFilters = document.querySelectorAll('.type-filter') as NodeListOf<HTMLInputElement>;
    const expandAllBtn = document.getElementById('expand-all');
    const collapseAllBtn = document.getElementById('collapse-all');
    const expandOneLevelBtn = document.getElementById('expand-one-level');
    const collapseOneLevelBtn = document.getElementById('collapse-one-level');

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

    // Expand one level button
    if (expandOneLevelBtn) {
        expandOneLevelBtn.addEventListener('click', () => {
            const currentDepth = getCurrentMaxDepth();
            expandToDepth(currentDepth + 1);
        });
    }

    // Collapse one level button  
    if (collapseOneLevelBtn) {
        collapseOneLevelBtn.addEventListener('click', () => {
            const currentDepth = getCurrentMaxDepth();
            collapseToDepth(currentDepth - 1);
        });
    }

    function getCurrentMaxDepth(): number {
        let maxDepth = 0;
        nodes.forEach(node => {
            const parentId = node.getAttribute('data-parent');
            if (!parentId) {
                maxDepth = Math.max(maxDepth, 0);
            } else if (expandedNodes.has(parentId)) {
                const currentDepth = parseInt(node.getAttribute('data-depth') || '0');
                maxDepth = Math.max(maxDepth, currentDepth);
            }
        });
        return maxDepth;
    }

    function expandToDepth(targetDepth: number) {
        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            
            if (id && hasChildren && depth < targetDepth) {
                expandedNodes.add(id);
                const toggleBtn = node.querySelector('.tree-toggle');
                if (toggleBtn) {
                    toggleBtn.classList.add('expanded');
                    const icon = toggleBtn.querySelector('.toggle-icon');
                    if (icon) icon.textContent = '−';
                }
            }
        });
        updateVisibility();
    }

    function collapseToDepth(targetDepth: number) {
        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            
            if (id && hasChildren && depth >= targetDepth) {
                expandedNodes.delete(id);
                const toggleBtn = node.querySelector('.tree-toggle');
                if (toggleBtn) {
                    toggleBtn.classList.remove('expanded');
                    const icon = toggleBtn.querySelector('.toggle-icon');
                    if (icon) icon.textContent = '+';
                }
            }
        });
        updateVisibility();
    }

    // Initialize visibility
    updateVisibility();
}
