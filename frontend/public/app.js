// Theme switching - run immediately to prevent flash
(function() {
    const stored = localStorage.getItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const theme = stored || (prefersDark ? 'dark' : 'light');
    document.documentElement.setAttribute('data-theme', theme);
})();

document.addEventListener('DOMContentLoaded', () => {
    // Theme toggle
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
        const updateIcon = () => {
            const current = document.documentElement.getAttribute('data-theme');
            themeToggle.textContent = current === 'dark' ? '☀️' : '🌙';
        };

        updateIcon();

        themeToggle.addEventListener('click', () => {
            const current = document.documentElement.getAttribute('data-theme');
            const next = current === 'dark' ? 'light' : 'dark';
            document.documentElement.setAttribute('data-theme', next);
            localStorage.setItem('theme', next);
            updateIcon();
        });
    }

    // Filtering logic - works on any element with data-filter-text
    const filterInput = document.getElementById('filter-input');

    if (filterInput) {
        filterInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase();
            const filterableItems = document.querySelectorAll('[data-filter-text]');

            filterableItems.forEach(item => {
                const text = item.getAttribute('data-filter-text');
                const matches = text && text.includes(query);

                // Determine appropriate display value based on element type
                if (matches) {
                    // Reset to default display
                    item.style.display = '';
                } else {
                    item.style.display = 'none';
                }
            });
        });
    }

    // Epic children toggle
    const toggleButtons = document.querySelectorAll('.toggle-children');
    toggleButtons.forEach(button => {
        button.addEventListener('click', () => {
            const epicItem = button.closest('.epic-item');
            const children = epicItem.querySelector('.epic-children');

            if (children) {
                const isCollapsed = children.classList.contains('collapsed');
                children.classList.toggle('collapsed');
                button.classList.toggle('expanded');

                if (isCollapsed) {
                    // Set max-height for animation
                    children.style.maxHeight = children.scrollHeight + 'px';
                    children.style.opacity = '1';
                } else {
                    children.style.maxHeight = '0';
                    children.style.opacity = '0';
                }
            }
        });
    });

    // Inline Editing Logic
    document.addEventListener('click', (e) => {
        if (e.target.classList.contains('issue-title')) {
            handleTitleEdit(e.target);
        }
    });

    function handleTitleEdit(titleEl) {
        const currentTitle = titleEl.innerText;
        const input = document.createElement('input');
        input.type = 'text';
        input.value = currentTitle;
        input.classList.add('edit-input');
        
        // Prevent click propagation to avoid immediate blur if we had a click listener on document
        input.addEventListener('click', (e) => e.stopPropagation());

        titleEl.replaceWith(input);
        input.focus();

        let isSaving = false;

        const save = async () => {
            if (isSaving) return;
            isSaving = true;

            const newTitle = input.value.trim();
            const issueItem = input.closest('.issue-item');
            const id = issueItem ? issueItem.getAttribute('data-id') : null;

            if (newTitle && newTitle !== currentTitle && id) {
                try {
                    const res = await fetch(`/api/issues/${id}`, {
                        method: 'POST',
                        headers: {'Content-Type': 'application/json'},
                        body: JSON.stringify({title: newTitle})
                    });
                    
                    if (!res.ok) throw new Error('Update failed');

                    // Restore title element with new value
                    const newTitleEl = document.createElement('div');
                    newTitleEl.classList.add('issue-title');
                    newTitleEl.innerText = newTitle;
                    input.replaceWith(newTitleEl);
                    
                    // Update filter text
                    const filterText = issueItem.getAttribute('data-filter-text');
                    if (filterText) {
                         // Simple replace might be risky if title is substring of status/id, but acceptable for now
                         // Better to reconstruct it if we had all data.
                         // Or just update the title part?
                         // Let's just leave it stale or reload page. Reloading is safest but disruptive.
                         // We'll leave it stale for filter for now.
                    }
                } catch (err) {
                    console.error(err);
                    alert('Failed to update title');
                    replaceWithOriginal();
                }
            } else {
                replaceWithOriginal();
            }
        };

        const replaceWithOriginal = () => {
            const originalTitleEl = document.createElement('div');
            originalTitleEl.classList.add('issue-title');
            originalTitleEl.innerText = currentTitle;
            input.replaceWith(originalTitleEl);
        };

        input.addEventListener('blur', save);
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                input.blur();
            } else if (e.key === 'Escape') {
                replaceWithOriginal();
                isSaving = true; // Prevent save on blur
            }
        });
    }

    // Column Visibility Toggle
    const columnsToggle = document.getElementById('columns-toggle');
    const columnsDropdown = document.getElementById('columns-dropdown');
    
    if (columnsToggle && columnsDropdown) {
        // Load saved visibility state
        const savedVisibility = localStorage.getItem('board-column-visibility');
        let visibilityState = savedVisibility ? JSON.parse(savedVisibility) : {};
        
        // Initialize checkboxes and apply visibility
        const columnCheckboxes = columnsDropdown.querySelectorAll('input[type="checkbox"]');
        columnCheckboxes.forEach(checkbox => {
            const status = checkbox.getAttribute('data-status');
            
            // Set checkbox state from saved state, default to checked
            checkbox.checked = visibilityState[status] !== false;
            
            // Apply initial visibility
            updateColumnVisibility(status, checkbox.checked);
            
            // Listen for changes
            checkbox.addEventListener('change', (e) => {
                const status = e.target.getAttribute('data-status');
                const isVisible = e.target.checked;
                
                updateColumnVisibility(status, isVisible);
                saveVisibilityState();
            });
        });
        
        function updateColumnVisibility(status, isVisible) {
            const column = document.querySelector(`.board-column[data-status="${status}"]`);
            if (column) {
                column.style.display = isVisible ? '' : 'none';
            }
        }
        
        function saveVisibilityState() {
            const newState = {};
            columnCheckboxes.forEach(checkbox => {
                const status = checkbox.getAttribute('data-status');
                newState[status] = checkbox.checked;
            });
            localStorage.setItem('board-column-visibility', JSON.stringify(newState));
        }
        
        // Toggle dropdown
        columnsToggle.addEventListener('click', (e) => {
            e.stopPropagation();
            columnsDropdown.classList.toggle('show');
        });
        
        // Close dropdown when clicking outside
        document.addEventListener('click', (e) => {
            if (!columnsDropdown.contains(e.target) && e.target !== columnsToggle) {
                columnsDropdown.classList.remove('show');
            }
        });
        
        // Prevent dropdown close when clicking inside
        columnsDropdown.addEventListener('click', (e) => {
            e.stopPropagation();
        });
    }

    // Board Drag and Drop
    const draggables = document.querySelectorAll('.issue-card[draggable="true"]');
    const droppables = document.querySelectorAll('.column-content');

    if (draggables.length > 0 && droppables.length > 0) {
        draggables.forEach(draggable => {
            draggable.addEventListener('dragstart', () => {
                draggable.classList.add('dragging');
                draggable.style.opacity = '0.5';
            });

            draggable.addEventListener('dragend', () => {
                draggable.classList.remove('dragging');
                draggable.style.opacity = '1';
            });
        });

        droppables.forEach(droppable => {
            droppable.addEventListener('dragover', e => {
                e.preventDefault();
                const afterElement = getDragAfterElement(droppable, e.clientY);
                const draggable = document.querySelector('.dragging');
                if (draggable) {
                    if (afterElement == null) {
                        droppable.appendChild(draggable);
                    } else {
                        droppable.insertBefore(draggable, afterElement);
                    }
                }
            });

            droppable.addEventListener('drop', async () => {
                const draggable = document.querySelector('.dragging');
                if (!draggable) return;

                const apiStatus = droppable.getAttribute('data-status');
                const id = draggable.getAttribute('data-id');

                if (id && apiStatus) {
                    try {
                        const res = await fetch(`/api/issues/${id}`, {
                            method: 'POST',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({status: apiStatus})
                        });
                        if (!res.ok) throw new Error('Update failed');
                    } catch (err) {
                        console.error(err);
                        alert('Failed to update status');
                        window.location.reload();
                    }
                }
            });
        });
    }

    function getDragAfterElement(container, y) {
        const draggableElements = [...container.querySelectorAll('.issue-card:not(.dragging)')];

        return draggableElements.reduce((closest, child) => {
            const box = child.getBoundingClientRect();
            const offset = y - box.top - box.height / 2;
            if (offset < 0 && offset > closest.offset) {
                return { offset: offset, element: child };
            } else {
                return closest;
            }
        }, { offset: Number.NEGATIVE_INFINITY }).element;
    }

    // Keyboard Navigation
    initKeyboardNavigation();

    function initKeyboardNavigation() {
        let selectedIndex = -1;
        let selectedColumnIndex = 0;
        let selectedCardIndex = 0;
        
        // Detect View Type
        const isBoard = document.querySelector('.board') !== null;
        const isList = document.querySelector('.issue-list') !== null;
        
        if (!isBoard && !isList) return;

        // Initial selection if in board mode
        if (isBoard) {
            updateBoardSelection();
        }

        document.addEventListener('keydown', (e) => {
            // Ignore if input is focused
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

            // Backspace navigation
            if (e.key === 'Backspace') {
                e.preventDefault();
                window.history.back();
                return;
            }

            if (isList) {
                handleListNavigation(e);
            } else if (isBoard) {
                handleBoardNavigation(e);
            }
        });

        function handleListNavigation(e) {
            const items = Array.from(document.querySelectorAll('.issue-item:not([style*="display: none"])'));
            if (items.length === 0) return;

            // Find currently selected
            const current = document.querySelector('.issue-item.selected');
            if (current) {
                selectedIndex = items.indexOf(current);
            }

            if (e.key === 'j' || e.key === 'ArrowDown') {
                selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
                selectItem(items[selectedIndex]);
                e.preventDefault();
            } else if (e.key === 'k' || e.key === 'ArrowUp') {
                selectedIndex = Math.max(selectedIndex - 1, 0);
                selectItem(items[selectedIndex]);
                e.preventDefault();
            } else if (e.key === 'Enter' || e.key === 'o') {
                if (current) {
                    const link = current.querySelector('.issue-meta a');
                    if (link) link.click();
                }
            }
        }

        function handleBoardNavigation(e) {
            const columns = Array.from(document.querySelectorAll('.board-column:not([style*="display: none"])'));
            if (columns.length === 0) return;

            if (e.key === 'j' || e.key === 'ArrowDown') {
                const col = columns[selectedColumnIndex];
                const cards = getVisibleCards(col);
                if (cards.length > 0) {
                    selectedCardIndex = Math.min(selectedCardIndex + 1, cards.length - 1);
                    updateBoardSelection();
                    e.preventDefault();
                }
            } else if (e.key === 'k' || e.key === 'ArrowUp') {
                selectedCardIndex = Math.max(selectedCardIndex - 1, 0);
                updateBoardSelection();
                e.preventDefault();
            } else if (e.key === 'h' || e.key === 'ArrowLeft') {
                selectedColumnIndex = Math.max(selectedColumnIndex - 1, 0);
                // Try to maintain relative vertical position or reset? Resetting is safer.
                // Or try to clamp to new column height
                const col = columns[selectedColumnIndex];
                const cards = getVisibleCards(col);
                selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
                updateBoardSelection();
                e.preventDefault();
            } else if (e.key === 'l' || e.key === 'ArrowRight') {
                selectedColumnIndex = Math.min(selectedColumnIndex + 1, columns.length - 1);
                const col = columns[selectedColumnIndex];
                const cards = getVisibleCards(col);
                selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
                updateBoardSelection();
                e.preventDefault();
            } else if (e.key === 'Enter' || e.key === 'o') {
                const selected = document.querySelector('.issue-card.selected');
                if (selected) {
                    const link = selected.querySelector('a');
                    if (link) link.click();
                }
            }
        }

        function getVisibleCards(column) {
             if (!column) return [];
             return Array.from(column.querySelectorAll('.issue-card:not([style*="display: none"])'));
        }

        function selectItem(item) {
            document.querySelectorAll('.issue-item.selected').forEach(el => el.classList.remove('selected'));
            if (item) {
                item.classList.add('selected');
                item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            }
        }

        function updateBoardSelection() {
            const columns = Array.from(document.querySelectorAll('.board-column:not([style*="display: none"])'));
            if (columns.length === 0) return;

            // Clamp column index
            selectedColumnIndex = Math.max(0, Math.min(selectedColumnIndex, columns.length - 1));
            const col = columns[selectedColumnIndex];
            
            const cards = getVisibleCards(col);
            
            document.querySelectorAll('.issue-card.selected').forEach(el => el.classList.remove('selected'));
            
            if (cards.length > 0) {
                // Clamp card index
                selectedCardIndex = Math.max(0, Math.min(selectedCardIndex, cards.length - 1));
                const card = cards[selectedCardIndex];
                card.classList.add('selected');
                card.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            } else {
                // No cards in this column, visual feedback on column header? 
                // For now, let's just ensure we don't crash.
                // Ideally, we might want to highlight the column itself.
            }
        }
    }
});
