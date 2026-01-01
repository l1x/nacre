/**
 * Dependency graph tree view - expand/collapse and epic selection
 */

export function initDependencyGraph() {
    const container = document.querySelector('.graph-tree-view');
    const epicSelect = document.getElementById('epic-select') as HTMLSelectElement | null;

    if (!container) return;

    // Epic selector - navigate to /graph/{epic_id}
    if (epicSelect) {
        epicSelect.addEventListener('change', () => {
            const epicId = epicSelect.value;
            if (epicId) {
                window.location.href = `/graph/${epicId}`;
            } else {
                window.location.href = '/graph';
            }
        });
    }

    // Tree expand/collapse
    container.addEventListener('click', (e) => {
        const toggle = (e.target as Element).closest('.tree-toggle');
        if (!toggle) return;

        const node = toggle.closest('.tree-node') as HTMLElement;
        const nodeId = node?.dataset.id;
        if (!nodeId) return;

        const isExpanded = toggle.classList.toggle('expanded');
        toggleDirectChildren(nodeId, isExpanded);
    });
}

/**
 * Toggle visibility of direct children of a node
 */
function toggleDirectChildren(parentId: string, show: boolean) {
    document.querySelectorAll(`[data-parent="${parentId}"]`).forEach(child => {
        child.classList.toggle('hidden', !show);

        // When hiding, also collapse any expanded children
        if (!show) {
            const childId = (child as HTMLElement).dataset.id;
            if (childId) {
                const toggle = child.querySelector('.tree-toggle');
                if (toggle?.classList.contains('expanded')) {
                    toggle.classList.remove('expanded');
                    toggleDirectChildren(childId, false);
                }
            }
        }
    });
}
