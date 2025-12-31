export function initNavigation() {
    let selectedIndex = -1;
    let selectedColumnIndex = 0;
    let selectedCardIndex = 0;
    
    const isBoard = document.querySelector('.board') !== null;
    const isList = document.querySelector('.issue-list') !== null;
    
    if (isBoard) {
        updateBoardSelection();
    }

    document.addEventListener('keydown', (e) => {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

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

    function handleListNavigation(e: KeyboardEvent) {
        const items = Array.from(document.querySelectorAll('.issue-item:not([style*="display: none"])')) as HTMLElement[];
        if (items.length === 0) return;

        const current = document.querySelector('.issue-item.selected') as HTMLElement | null;
        if (current) {
            selectedIndex = items.indexOf(current);
        }

        if (e.key === 'j' || e.key === 'ArrowDown') {
            selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
            const item = items.at(selectedIndex);
            if (item) selectItem(item);
            e.preventDefault();
        } else if (e.key === 'k' || e.key === 'ArrowUp') {
            selectedIndex = Math.max(selectedIndex - 1, 0);
            const item = items.at(selectedIndex);
            if (item) selectItem(item);
            e.preventDefault();
        } else if (e.key === 'Enter' || e.key === 'o') {
            if (current) {
                const link = current.querySelector('.issue-meta a') as HTMLElement | null;
                if (link) link.click();
            }
        }
    }

    function handleBoardNavigation(e: KeyboardEvent) {
        const columns = Array.from(document.querySelectorAll('.board-column:not([style*="display: none"])')) as HTMLElement[];
        if (columns.length === 0) return;

        if (e.key === 'j' || e.key === 'ArrowDown') {
            const col = columns.at(selectedColumnIndex);
            if (!col) return;
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
            const col = columns.at(selectedColumnIndex);
            if (!col) return;
            const cards = getVisibleCards(col);
            selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
            updateBoardSelection();
            e.preventDefault();
        } else if (e.key === 'l' || e.key === 'ArrowRight') {
            selectedColumnIndex = Math.min(selectedColumnIndex + 1, columns.length - 1);
            const col = columns.at(selectedColumnIndex);
            if (!col) return;
            const cards = getVisibleCards(col);
            selectedCardIndex = Math.min(selectedCardIndex, Math.max(0, cards.length - 1));
            updateBoardSelection();
            e.preventDefault();
        } else if (e.key === 'Enter' || e.key === 'o') {
            const selected = document.querySelector('.issue-card.selected') as HTMLElement | null;
            if (selected) {
                const link = selected.querySelector('a') as HTMLElement | null;
                if (link) link.click();
            }
        }
    }

    function getVisibleCards(column: HTMLElement): HTMLElement[] {
         if (!column) return [];
         return Array.from(column.querySelectorAll('.issue-card:not([style*="display: none"])')) as HTMLElement[];
    }

    function selectItem(item: HTMLElement) {
        document.querySelectorAll('.issue-item.selected').forEach(el => el.classList.remove('selected'));
        if (item) {
            item.classList.add('selected');
            item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
    }

    function updateBoardSelection() {
        const columns = Array.from(document.querySelectorAll('.board-column:not([style*="display: none"])')) as HTMLElement[];
        if (columns.length === 0) return;

        selectedColumnIndex = Math.max(0, Math.min(selectedColumnIndex, columns.length - 1));
        const col = columns.at(selectedColumnIndex);
        if (!col) return;
        const cards = getVisibleCards(col);

        document.querySelectorAll('.issue-card.selected').forEach(el => el.classList.remove('selected'));

        if (cards.length > 0) {
            selectedCardIndex = Math.max(0, Math.min(selectedCardIndex, cards.length - 1));
            const card = cards.at(selectedCardIndex);
            if (card) {
                card.classList.add('selected');
                card.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            }
        }
    }
}
