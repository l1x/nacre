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
            expandOneLevel();
        });
    }

    // Collapse one level button  
    if (collapseOneLevelBtn) {
        collapseOneLevelBtn.addEventListener('click', () => {
            collapseOneLevel();
        });
    }

    function expandOneLevel() {
        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            const parentId = node.getAttribute('data-parent');
            
            // Find all expanded nodes at the previous level to determine current max depth
            let maxDepth = -1;
            nodes.forEach(n => {
                if (n.getAttribute('data-parent') === '0') { // Root nodes
                    maxDepth = Math.max(maxDepth, 0);
                } else if (expandedNodes.has(n.getAttribute('data-parent'))) {
                    const nodeDepth = parseInt(n.getAttribute('data-depth') || '0');
                    maxDepth = Math.max(maxDepth, nodeDepth - 1);
                }
            });
            
            // Expand nodes at the next level if their parents are visible
            if (id && hasChildren && depth === maxDepth + 1) {
                // Check if parent is visible (either root or expanded)
                const parentVisible = !parentId || expandedNodes.has(parentId);
                if (parentVisible) {
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
    }

    function collapseOneLevel() {
        // Find current maximum expanded depth
        let maxExpandedDepth = 0;
        expandedNodes.forEach(id => {
            nodes.forEach(node => {
                if (node.getAttribute('data-id') === id) {
                    const depth = parseInt(node.getAttribute('data-depth') || '0');
                    maxExpandedDepth = Math.max(maxExpandedDepth, depth);
                }
            });
        });
        
        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            
            // Collapse nodes deeper than the new max depth
            if (id && hasChildren && depth > maxExpandedDepth - 1) {
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

    function getCurrentExpandedDepths(): number[] {
        const depths: number[] = [];
        nodes.forEach(node => {
            const id = node.getAttribute('data-id');
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            if (id && expandedNodes.has(id)) {
                depths.push(depth);
            }
        });
        return [...new Set(depths)];
    }

    function isParentVisible(node: Element): boolean {
        const parentId = node.getAttribute('data-parent');
        if (!parentId) return true; // Root node
        
        const parent = Array.from(nodes).find(n => 
            n.getAttribute('data-id') === parentId
        );
        if (!parent) return false;
        
        // Check if parent is visible (not hidden by filters and has expanded parent)
        return !parent.classList.contains('hidden') && isParentVisible(parent);
    }

    // Initialize visibility
    updateVisibility();
}
