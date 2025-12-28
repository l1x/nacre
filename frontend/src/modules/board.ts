export function initBoardFeatures() {
    // Column Visibility Toggle
    const columnsToggle = document.getElementById('columns-toggle');
    const columnsDropdown = document.getElementById('columns-dropdown');
    
    if (columnsToggle && columnsDropdown) {
        const savedVisibility = localStorage.getItem('board-column-visibility');
        let visibilityState = savedVisibility ? JSON.parse(savedVisibility) : null;
        
        const columnCheckboxes = columnsDropdown.querySelectorAll('input[type="checkbox"]') as NodeListOf<HTMLInputElement>;
        columnCheckboxes.forEach(checkbox => {
            const status = checkbox.getAttribute('data-status');
            if (!status) return;
            
            // Default: deferred is hidden, others are visible
            if (visibilityState === null) {
                checkbox.checked = status !== 'deferred';
            } else {
                checkbox.checked = visibilityState[status] !== false;
            }
            
            updateColumnVisibility(status, checkbox.checked);
            
            checkbox.addEventListener('change', (e) => {
                const isVisible = (e.target as HTMLInputElement).checked;
                updateColumnVisibility(status, isVisible);
                saveVisibilityState();
            });
        });
        
        function updateColumnVisibility(status: string, isVisible: boolean) {
            const column = document.querySelector(`.board-column[data-status="${status}"]`) as HTMLElement | null;
            if (column) {
                column.style.display = isVisible ? '' : 'none';
            }
        }
        
        function saveVisibilityState() {
            const newState: Record<string, boolean> = {};
            columnCheckboxes.forEach(checkbox => {
                const status = checkbox.getAttribute('data-status');
                if (status) newState[status] = checkbox.checked;
            });
            localStorage.setItem('board-column-visibility', JSON.stringify(newState));
        }
        
        columnsToggle.addEventListener('click', (e) => {
            e.stopPropagation();
            columnsDropdown.classList.toggle('show');
        });
        
        document.addEventListener('click', (e) => {
            if (!columnsDropdown.contains(e.target as Node) && e.target !== columnsToggle) {
                columnsDropdown.classList.remove('show');
            }
        });
        
        columnsDropdown.addEventListener('click', (e) => {
            e.stopPropagation();
        });
    }

    // Type Filtering
    const typeFilters = document.querySelectorAll('.type-filter') as NodeListOf<HTMLInputElement>;
    if (typeFilters.length > 0) {
        const updateCardVisibility = () => {
            const activeTypes = new Set(
                Array.from(typeFilters)
                    .filter(f => f.checked)
                    .map(f => f.value)
            );

            const cards = document.querySelectorAll('.issue-card') as NodeListOf<HTMLElement>;
            cards.forEach(card => {
                let visible = false;
                for (const type of activeTypes) {
                    if (card.classList.contains(`issue-type-${type}`)) {
                        visible = true;
                        break;
                    }
                }
                
                if (visible) {
                    card.classList.remove('hidden-by-type');
                } else {
                    card.classList.add('hidden-by-type');
                }
            });
        };

        typeFilters.forEach(filter => {
            filter.addEventListener('change', updateCardVisibility);
        });
        
        updateCardVisibility();
    }
}
