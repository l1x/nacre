class NacreApp extends HTMLElement {
    constructor() {
        super();
        this.issues = [];
    }

    connectedCallback() {
        this.innerHTML = `
            <header>
                <h1>Nacre</h1>
                <nav>
                    <a href="#" class="active">Issues</a>
                    <a href="#">Epics</a>
                    <a href="#">Board</a>
                </nav>
            </header>
            <main id="content">
                <div class="loading">Loading...</div>
            </main>
        `;
        this.loadIssues();
    }

    async loadIssues() {
        try {
            const response = await fetch('/api/issues');
            this.issues = await response.json();
            this.renderIssues();
        } catch (e) {
            console.error('Failed to load issues', e);
            this.querySelector('#content').innerHTML = '<div class="error">Failed to load issues</div>';
        }
    }

    renderIssues() {
        const content = this.querySelector('#content');
        if (this.issues.length === 0) {
            content.innerHTML = '<div class="empty">No issues found</div>';
            return;
        }

        const listHtml = this.issues.map(issue => `
            <div class="issue-item">
                <div class="issue-main">
                    <div class="issue-title">${this.escapeHtml(issue.title)}</div>
                    <div class="issue-meta">${issue.id} • ${issue.issue_type} • P${issue.priority || '-'}</div>
                </div>
                <div class="issue-status">
                    <span class="status-badge">${issue.status}</span>
                </div>
            </div>
        `).join('');

        content.innerHTML = `<div class="issue-list">${listHtml}</div>`;
    }

    escapeHtml(unsafe) {
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }
}

customElements.define('nacre-app', NacreApp);
