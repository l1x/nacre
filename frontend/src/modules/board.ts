export function initBoardFeatures() {
    // Column Visibility Toggle
    const columnsToggle = document.getElementById('columns-toggle');
    const columnsDropdown = document.getElementById('columns-dropdown');

    if (columnsToggle && columnsDropdown) {
        const savedVisibility = localStorage.getItem('board-column-visibility');
        let visibilityState = savedVisibility ? JSON.parse(savedVisibility) : null;

        // Helper to get checkboxes (supports dynamic DOM)
        const getColumnCheckboxes = () =>
            columnsDropdown.querySelectorAll('input[type="checkbox"]') as NodeListOf<HTMLInputElement>;

        // Initialize visibility state
        getColumnCheckboxes().forEach((checkbox) => {
            const status = checkbox.getAttribute('data-status');
            if (!status) return;

            // Default: deferred is hidden, others are visible
            if (visibilityState === null) {
                checkbox.checked = status !== 'deferred';
            } else {
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
            const newState: Record<string, boolean> = {};
            getColumnCheckboxes().forEach((checkbox) => {
                const status = checkbox.getAttribute('data-status');
                if (status) newState[status] = checkbox.checked;
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

    // Type Filtering - use event delegation on document for type-filter changes
    const updateCardVisibility = () => {
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

    // Event delegation: type filters (handled at document level to catch all filters)
    document.addEventListener('change', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('type-filter')) {
            updateCardVisibility();
        }
    });

    // Initial visibility update
    updateCardVisibility();
}
