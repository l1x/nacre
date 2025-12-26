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
}
