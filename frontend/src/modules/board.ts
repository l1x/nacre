import { STATUS, Status } from '../constants';

const VALID_STATUSES = Object.values(STATUS);

function isValidStatus(value: string): value is Status {
    return VALID_STATUSES.includes(value as Status);
}

export function initBoardFeatures() {
    // Column Visibility Toggle
    const columnsToggle = document.getElementById('columns-toggle');
    const columnsDropdown = document.getElementById('columns-dropdown');

    if (columnsToggle && columnsDropdown) {
        const savedVisibility = localStorage.getItem('board-column-visibility');
        const visibilityState = savedVisibility ? JSON.parse(savedVisibility) : null;

        // Helper to get checkboxes (supports dynamic DOM)
        const getColumnCheckboxes = () =>
            columnsDropdown.querySelectorAll('input[type="checkbox"]') as NodeListOf<HTMLInputElement>;

        // Initialize visibility state
        getColumnCheckboxes().forEach((checkbox) => {
            const status = checkbox.getAttribute('data-status');
            if (!status || !isValidStatus(status)) return;

            // Default: deferred is hidden, others are visible
            if (visibilityState === null) {
                checkbox.checked = status !== STATUS.DEFERRED;
            } else {
                // Safe: status validated via isValidStatus(), data from user's own localStorage
                // eslint-disable-next-line security/detect-object-injection
                checkbox.checked = visibilityState[status] !== false;
            }

            updateColumnVisibility(status, checkbox.checked);
        });

        function updateColumnVisibility(status: string, isVisible: boolean) {
            const column = document.querySelector(`.board-column[data-status="${status}"]`) as HTMLElement | null;
            if (column) {
                column.style.display = isVisible ? '' : 'none';
            }
        }

        function saveVisibilityState() {
            const newState: Record<Status, boolean> = {} as Record<Status, boolean>;
            getColumnCheckboxes().forEach((checkbox) => {
                const status = checkbox.getAttribute('data-status');
                // Safe: status validated via isValidStatus() before using as object key
                if (status && isValidStatus(status)) {
                    // eslint-disable-next-line security/detect-object-injection
                    newState[status] = checkbox.checked;
                }
            });
            localStorage.setItem('board-column-visibility', JSON.stringify(newState));
        }

        // Event delegation: handle checkbox changes within dropdown
        columnsDropdown.addEventListener('change', (e) => {
            const target = e.target as HTMLInputElement;
            if (target.type !== 'checkbox') return;

            const status = target.getAttribute('data-status');
            if (!status) return;

            updateColumnVisibility(status, target.checked);
            saveVisibilityState();
        });

        columnsToggle.addEventListener('click', (e) => {
            e.stopPropagation();
            columnsDropdown.classList.toggle('show');
        });

        // Global click to close dropdown (event delegation on document)
        document.addEventListener('click', (e) => {
            if (!columnsDropdown.contains(e.target as Node) && e.target !== columnsToggle) {
                columnsDropdown.classList.remove('show');
            }
        });

        columnsDropdown.addEventListener('click', (e) => {
            e.stopPropagation();
        });
    }

    // --- Type Filtering ---
    const updateTypeVisibility = () => {
        const typeFilters = document.querySelectorAll('.type-filter') as NodeListOf<HTMLInputElement>;
        if (typeFilters.length === 0) return;

        const activeTypes = new Set(
            Array.from(typeFilters)
                .filter((f) => f.checked)
                .map((f) => f.value)
        );

        const cards = document.querySelectorAll('.issue-card') as NodeListOf<HTMLElement>;
        cards.forEach((card) => {
            let visible = false;
            for (const type of activeTypes) {
                if (card.classList.contains(`issue-type-${type}`)) {
                    visible = true;
                    break;
                }
            }

            card.classList.toggle('hidden-by-type', !visible);
        });
    };

    // --- Priority Filtering ---
    const updatePriorityVisibility = () => {
        const priorityFilters = document.querySelectorAll('.priority-filter') as NodeListOf<HTMLInputElement>;
        if (priorityFilters.length === 0) return;

        const activePriorities = new Set(
            Array.from(priorityFilters)
                .filter((f) => f.checked)
                .map((f) => f.value)
        );

        const cards = document.querySelectorAll('.issue-card') as NodeListOf<HTMLElement>;
        cards.forEach((card) => {
            const priority = card.getAttribute('data-priority') || '0';
            card.classList.toggle('hidden-by-priority', !activePriorities.has(priority));
        });
    };

    // --- Assignee Filtering ---
    const updateAssigneeVisibility = () => {
        const assigneeFilters = document.querySelectorAll('.assignee-filter') as NodeListOf<HTMLInputElement>;
        if (assigneeFilters.length === 0) return;

        const activeAssignees = new Set(
            Array.from(assigneeFilters)
                .filter((f) => f.checked)
                .map((f) => f.value)
        );

        const cards = document.querySelectorAll('.issue-card') as NodeListOf<HTMLElement>;
        cards.forEach((card) => {
            const assignee = card.getAttribute('data-assignee') || '';
            card.classList.toggle('hidden-by-assignee', !activeAssignees.has(assignee));
        });
    };

    // --- Sort Within Columns ---
    const sortColumns = (sortBy: string) => {
        const columns = document.querySelectorAll('.column-content') as NodeListOf<HTMLElement>;
        columns.forEach((column) => {
            const cards = Array.from(column.querySelectorAll('.issue-card')) as HTMLElement[];

            cards.sort((a, b) => {
                if (sortBy === 'priority') {
                    const pa = parseInt(a.getAttribute('data-priority') || '0');
                    const pb = parseInt(b.getAttribute('data-priority') || '0');
                    return pa - pb;
                } else if (sortBy === 'created') {
                    const ca = parseInt(a.getAttribute('data-created') || '0');
                    const cb = parseInt(b.getAttribute('data-created') || '0');
                    return cb - ca; // newest first
                } else {
                    // alphabetical by title
                    const ta = a.querySelector('.issue-title')?.textContent?.trim() || '';
                    const tb = b.querySelector('.issue-title')?.textContent?.trim() || '';
                    return ta.localeCompare(tb);
                }
            });

            // Re-append in sorted order
            cards.forEach((card) => column.appendChild(card));
        });
    };

    // Event delegation: all filter changes
    document.addEventListener('change', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('type-filter')) {
            updateTypeVisibility();
        } else if (target.classList.contains('priority-filter')) {
            updatePriorityVisibility();
        } else if (target.classList.contains('assignee-filter')) {
            updateAssigneeVisibility();
        }
    });

    // Sort select
    const sortSelect = document.getElementById('board-sort') as HTMLSelectElement | null;
    if (sortSelect) {
        sortSelect.addEventListener('change', () => {
            sortColumns(sortSelect.value);
        });
    }

    // Initial updates
    updateTypeVisibility();
    updatePriorityVisibility();
    updateAssigneeVisibility();
    sortColumns('priority'); // default sort
}
