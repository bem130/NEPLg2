export class EditorDOMUI {
    constructor(editor, customElements = {}) {
        this.editor = editor;
        this.completionList = customElements.completionList || document.getElementById('completion-list');
        this.popup = customElements.popup || document.getElementById('general-popup'); // Hover popup
        this.problemsPanel = customElements.problemsPanel || null;

        this.isCompletionVisible = false;
        this.completionSuggestions = [];
        this.selectedSuggestionIndex = 0;
    }

    showHover(hoverInfo, x, y) {
        if (!this.popup) return;

        // Convert markdown to HTML? Simple text for now
        this.popup.innerHTML = hoverInfo.content.replace(/\n/g, '<br>').replace(/\*\*(.*?)\*\*/g, '<b>$1</b>');

        this.popup.style.display = 'block';
        this.popup.classList.remove('hidden');

        const canvasRect = this.editor.canvas.getBoundingClientRect();
        this.popup.style.left = `${canvasRect.left + x + 10}px`;
        this.popup.style.top = `${canvasRect.top + y + 10}px`;
    }

    hideHover() {
        if (this.popup) {
            this.popup.style.display = 'none';
            this.popup.classList.add('hidden');
        }
    }

    showCompletion(suggestions) {
        this.completionSuggestions = suggestions;
        this.isCompletionVisible = true;
        this.selectedSuggestionIndex = 0;

        if (!this.completionList) return;

        this.completionList.innerHTML = '';
        suggestions.forEach((item, index) => {
            const li = document.createElement('li');
            li.textContent = item.label;
            li.className = index === 0 ? 'selected' : '';
            li.addEventListener('click', () => {
                this.selectedSuggestionIndex = index;
                this.editor.acceptCompletion();
            });
            this.completionList.appendChild(li);
        });

        this.completionList.classList.remove('hidden');
        this.completionList.style.display = 'block';

        this.editor.updateTextareaPosition(); // Reposition to cursor
    }

    hideCompletion() {
        this.isCompletionVisible = false;
        if (this.completionList) {
            this.completionList.classList.add('hidden');
            this.completionList.style.display = 'none';
        }
    }

    updateProblemsPanel() {
        // ...
    }
}
