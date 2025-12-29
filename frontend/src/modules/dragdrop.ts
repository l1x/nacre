import { handleError, handleNetworkError } from './toast';

export function initDragAndDrop() {
    const draggables = document.querySelectorAll('.issue-card[draggable="true"]') as NodeListOf<HTMLElement>;
    const droppables = document.querySelectorAll('.column-content') as NodeListOf<HTMLElement>;

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
                const dragging = document.querySelector('.dragging') as HTMLElement | null;
                if (dragging) {
                    const afterElement = getDragAfterElement(droppable, e.clientY);
                    if (afterElement == null) {
                        droppable.appendChild(dragging);
                    } else {
                        droppable.insertBefore(dragging, afterElement);
                    }
                }
            });

            droppable.addEventListener('drop', async () => {
                const draggable = document.querySelector('.dragging') as HTMLElement | null;
                if (!draggable) return;

                const apiStatus = droppable.getAttribute('data-status');
                const id = draggable.getAttribute('data-id');

                if (id && apiStatus) {
                    const updateStatus = async () => {
                        const res = await fetch(`/api/issues/${id}`, {
                            method: 'POST',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({status: apiStatus})
                        });
                        if (!res.ok) throw new Error('Update failed');
                    };

                    try {
                        await updateStatus();
                    } catch (err) {
                        if (err instanceof Error && err.message === 'Update failed') {
                            handleNetworkError(new Response(null, { status: 500, statusText: 'Update failed' }), 'Failed to update status', updateStatus);
                        } else {
                            handleError(err, 'Failed to update status', updateStatus);
                        }
                    }
                }
            });
        });
    }

    function getDragAfterElement(container: HTMLElement, y: number): HTMLElement | null {
        const draggableElements = [...container.querySelectorAll('.issue-card:not(.dragging)')] as HTMLElement[];

        const result = draggableElements.reduce((closest, child) => {
            const box = child.getBoundingClientRect();
            const offset = y - box.top - box.height / 2;
            if (offset < 0 && offset > closest.offset) {
                return { offset: offset, element: child };
            } else {
                return closest;
            }
        }, { offset: Number.NEGATIVE_INFINITY, element: null as HTMLElement | null });
        
        return result.element;
    }
}
