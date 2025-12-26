export function initInlineEdit() {
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        // Only allow inline editing for titles in the list view (issue-item)
        if (target.classList.contains('issue-title') && target.closest('.issue-item')) {
            handleTitleEdit(target);
        }
    });

    function handleTitleEdit(titleEl: HTMLElement) {
        const currentTitle = titleEl.innerText;
        const input = document.createElement('input');
        input.type = 'text';
        input.value = currentTitle;
        input.classList.add('edit-input');
        
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

                    const newTitleEl = document.createElement('div');
                    newTitleEl.classList.add('issue-title');
                    newTitleEl.innerText = newTitle;
                    input.replaceWith(newTitleEl);
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
                isSaving = true;
            }
        });
    }
}
