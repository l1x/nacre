import { ISSUE_TYPE } from '../constants';

function initEpicSelector() {
    const wrapper = document.querySelector('.epic-selector-wrapper');
    if (!wrapper) return;

    const selector = wrapper.querySelector('.epic-selector') as HTMLElement;
    const leftBtn = wrapper.querySelector('#epic-nav-left') as HTMLButtonElement;
    const rightBtn = wrapper.querySelector('#epic-nav-right') as HTMLButtonElement;

    if (!selector || !leftBtn || !rightBtn) return;

    const scrollAmount = 200; // pixels to scroll per click

    const updateButtonStates = () => {
        const { scrollLeft, scrollWidth, clientWidth } = selector;
        leftBtn.disabled = scrollLeft <= 0;
        rightBtn.disabled = scrollLeft + clientWidth >= scrollWidth - 1;
    };

    leftBtn.addEventListener('click', () => {
        selector.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
    });

    rightBtn.addEventListener('click', () => {
        selector.scrollBy({ left: scrollAmount, behavior: 'smooth' });
    });

    selector.addEventListener('scroll', updateButtonStates);
    window.addEventListener('resize', updateButtonStates);

    // Initial state
    updateButtonStates();
}

export function initGraph() {
    // Initialize epic selector pagination
    initEpicSelector();

    const treeView = document.querySelector('.tree-view');
    if (!treeView) return;

    const treeList = treeView.querySelector('.tree-list');
    const controlsContainer = document.querySelector('.controls-grid') || document.querySelector('.child-expand-controls');
    const expandedNodes = new Set<string>();

    // Helper to get current nodes (supports dynamic DOM)
    const getNodes = () => treeView.querySelectorAll('.tree-node') as NodeListOf<HTMLElement>;
    const getTypeFilters = () => document.querySelectorAll('.type-filter') as NodeListOf<HTMLInputElement>;

    // Initial expansion for task view
    const issueType = treeView.getAttribute('data-issue-type');
    if (issueType === ISSUE_TYPE.TASK) {
        getNodes().forEach(node => {
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
    }

    function updateVisibility() {
        const typeFilters = getTypeFilters();
        const activeTypes = typeFilters.length > 0
            ? new Set(Array.from(typeFilters).filter(f => f.checked).map(f => f.value))
            : null;

        getNodes().forEach(node => {
            const parentId = node.getAttribute('data-parent') || '';
            const type = node.getAttribute('data-type') || '';

            let visible = false;

            // Top-level nodes (no parent) are always visible
            if (!parentId) {
                visible = true;
            } else if (expandedNodes.has(parentId)) {
                // Children are visible if parent is expanded
                visible = true;
            }

            // Must also match type filter if filters exist
            if (visible && activeTypes && !activeTypes.has(type)) {
                visible = false;
            }

            node.classList.toggle('hidden', !visible);
        });
    }

    // Event delegation: single click handler on tree list for toggle buttons
    if (treeList) {
        treeList.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
            const toggleBtn = target.closest('.tree-toggle');
            if (!toggleBtn) return;

            e.preventDefault();
            e.stopPropagation();

            const node = toggleBtn.closest('.tree-node');
            if (!node) return;

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

    // Event delegation: type filters (on document to catch dynamically added filters)
    document.addEventListener('change', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('type-filter')) {
            updateVisibility();
        }
    });

    // Event delegation: control buttons
    if (controlsContainer) {
        controlsContainer.addEventListener('click', (e) => {
            const target = e.target as HTMLElement;
            const button = target.closest('button');
            if (!button) return;

            const id = button.id;
            if (id === 'expand-all' || id === 'detail-expand') {
                expandAll();
            } else if (id === 'collapse-all' || id === 'detail-collapse') {
                collapseAll();
            } else if (id === 'expand-one-level') {
                expandOneLevel();
            } else if (id === 'collapse-one-level') {
                collapseOneLevel();
            }
        });
    }

    function expandAll() {
        getNodes().forEach(node => {
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
    }

    function collapseAll() {
        expandedNodes.clear();
        getNodes().forEach(node => {
            const toggleBtn = node.querySelector('.tree-toggle');
            if (toggleBtn) {
                toggleBtn.classList.remove('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) icon.textContent = '+';
            }
        });
        updateVisibility();
    }

    function expandOneLevel() {
        const nodes = getNodes();
        // 1. Calculate the current maximum depth of expanded nodes.
        let currentMaxExpandedDepth = -1;
        nodes.forEach(node => {
            const id = node.getAttribute('data-id');
            if (id && expandedNodes.has(id)) {
                const depth = parseInt(node.getAttribute('data-depth') || '0');
                if (depth > currentMaxExpandedDepth) {
                    currentMaxExpandedDepth = depth;
                }
            }
        });

        // 2. The target depth to expand is one level deeper.
        const targetDepth = currentMaxExpandedDepth + 1;
        const nodesToExpand: { id: string; element: Element }[] = [];

        // 3. Identify nodes to expand
        nodes.forEach(node => {
            const depth = parseInt(node.getAttribute('data-depth') || '0');
            const id = node.getAttribute('data-id');
            const hasChildren = node.getAttribute('data-has-children') === 'true';
            const parentId = node.getAttribute('data-parent');

            if (id && hasChildren && !expandedNodes.has(id)) {
                if (depth <= targetDepth) {
                    // Check if parent is visible (either root or expanded)
                    const parentExpanded = !parentId || expandedNodes.has(parentId);
                    if (parentExpanded) {
                        nodesToExpand.push({ id, element: node });
                    }
                }
            }
        });

        // 4. Batch update
        nodesToExpand.forEach(({ id, element }) => {
            expandedNodes.add(id);
            const toggleBtn = element.querySelector('.tree-toggle');
            if (toggleBtn) {
                toggleBtn.classList.add('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) icon.textContent = '−';
            }
        });

        updateVisibility();
    }

    function collapseOneLevel() {
        const nodes = getNodes();
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

    // Initialize visibility
    updateVisibility();
}
