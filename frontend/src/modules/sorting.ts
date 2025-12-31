import { STATUS_ORDER, TYPE_ORDER, Status, IssueType } from '../constants';

interface SortConfig {
    status: string;
    type: string;
    priority: string;
}

const VALID_SORT_KEYS: readonly (keyof SortConfig)[] = ['status', 'type', 'priority'];

function isValidSortKey(key: string): key is keyof SortConfig {
    return VALID_SORT_KEYS.includes(key as keyof SortConfig);
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
    // Event delegation: handle sort button clicks at document level
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        const button = target.closest('.sort-btn');
        if (!button) return;

        // If clicking the already-active sort button, restore tree view
        if (button.classList.contains('active')) {
            restoreTreeView();
            button.classList.remove('active');
            return;
        }

        const sortBy = button.getAttribute('data-sort');
        if (sortBy && isValidSortKey(sortBy)) {
            sortTreeNodes(sortBy);
            updateActiveSortButton(button);
        }
    });
}

function restoreTreeView() {
    // Reload the page to restore original tree structure
    // This preserves any URL parameters (filters, etc.)
    window.location.reload();
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

    // Enable flat list view (removes tree indentation)
    treeList.classList.add('sorting-active');

    // Disable expand controls (they don't apply to flat view)
    const expandButtons = document.querySelectorAll('#expand-all, #collapse-all, #expand-one-level, #collapse-one-level');
    expandButtons.forEach(btn => {
        (btn as HTMLButtonElement).disabled = true;
    });

    // Clear and re-append sorted nodes
    treeList.innerHTML = '';
    nodes.forEach(node => {
        // Show all nodes (sorting flattens the tree)
        node.classList.remove('hidden');
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
    const orderA = STATUS_ORDER[statusA as Status] ?? 999;
    const orderB = STATUS_ORDER[statusB as Status] ?? 999;
    return orderA - orderB;
}

function compareType(typeA: string, typeB: string): number {
    const orderA = TYPE_ORDER[typeA as IssueType] ?? 999;
    const orderB = TYPE_ORDER[typeB as IssueType] ?? 999;
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
        button.classList.remove('active');
    });

    activeButton.classList.add('active');
}