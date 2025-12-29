interface SortConfig {
    status: string;
    type: string;
    priority: string;
}

interface TreeNode extends HTMLElement {
    dataset: {
        id: string;
        depth: string;
        type: string;
        status: string;
        priority: string;
        parent?: string;
        hasChildren: string;
        filterText: string;
    };
}

export function initSorting() {
    const sortButtons = document.querySelectorAll('.sort-btn');
    if (sortButtons.length === 0) return;

    sortButtons.forEach(button => {
        button.addEventListener('click', () => {
            const sortBy = button.getAttribute('data-sort');
            if (sortBy) {
                sortTreeNodes(sortBy);
                updateActiveSortButton(button);
            }
        });
    });
}

function sortTreeNodes(sortBy: keyof SortConfig) {
    const treeList = document.querySelector('.tree-list');
    if (!treeList) return;

    const nodes = Array.from(treeList.querySelectorAll('.tree-node')) as TreeNode[];
    
    // Store current expand/collapse state
    const expandedStates = new Map<string, boolean>();
    nodes.forEach(node => {
        const toggleBtn = node.querySelector('.tree-toggle') as HTMLButtonElement;
        if (toggleBtn && toggleBtn.classList.contains('expanded')) {
            expandedStates.set(node.dataset.id, true);
        }
    });

    // Sort nodes based on the selected criteria
    nodes.sort((a, b) => compareNodes(a, b, sortBy));

    // Clear and re-append sorted nodes
    treeList.innerHTML = '';
    nodes.forEach(node => {
        treeList.appendChild(node);
        
        // Restore expand/collapse state
        if (expandedStates.has(node.dataset.id)) {
            const toggleBtn = node.querySelector('.tree-toggle') as HTMLButtonElement;
            if (toggleBtn) {
                toggleBtn.classList.add('expanded');
                const icon = toggleBtn.querySelector('.toggle-icon');
                if (icon) {
                    icon.textContent = '−';
                }
            }
        }
    });
}

function compareNodes(a: TreeNode, b: TreeNode, sortBy: keyof SortConfig): number {
    switch (sortBy) {
        case 'status':
            return compareStatus(a.dataset.status, b.dataset.status);
        case 'type':
            return compareType(a.dataset.type, b.dataset.type);
        case 'priority':
            return comparePriority(a.dataset.priority, b.dataset.priority);
        default:
            return 0;
    }
}

function compareStatus(statusA: string, statusB: string): number {
    const statusOrder = { 'open': 0, 'in_progress': 1, 'blocked': 2, 'closed': 3, 'deferred': 4 };
    const orderA = statusOrder[statusA as keyof typeof statusOrder] ?? 999;
    const orderB = statusOrder[statusB as keyof typeof statusOrder] ?? 999;
    return orderA - orderB;
}

function compareType(typeA: string, typeB: string): number {
    const typeOrder = { 'epic': 0, 'feature': 1, 'bug': 2, 'task': 3, 'chore': 4 };
    const orderA = typeOrder[typeA as keyof typeof typeOrder] ?? 999;
    const orderB = typeOrder[typeB as keyof typeof typeOrder] ?? 999;
    return orderA - orderB;
}

function comparePriority(priorityA: string, priorityB: string): number {
    const prioA = parseInt(priorityA) || 999;
    const prioB = parseInt(priorityB) || 999;
    return prioA - prioB;
}

function updateActiveSortButton(activeButton: Element) {
    const sortButtons = document.querySelectorAll('.sort-btn');
    sortButtons.forEach(button => {
        button.classList.remove('btn-primary');
        button.classList.add('btn-tertiary');
    });
    
    activeButton.classList.remove('btn-tertiary');
    activeButton.classList.add('btn-primary');
}