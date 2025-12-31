interface ToastConfig {
    message: string;
    type?: 'error' | 'warning' | 'info' | 'success';
    duration?: number;
    retryAction?: () => Promise<void>;
}

class ToastManager {
    private container: HTMLElement | null = null;
    private activeToasts: Set<HTMLElement> = new Set();
    private initialized: boolean = false;

    constructor() {
        // Defer initialization until DOM is ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => this.init());
        } else {
            this.init();
        }
    }

    private init() {
        if (this.initialized) return;
        this.container = document.createElement('div');
        this.container.id = 'toast-container';
        this.container.className = 'toast-container';
        document.body.appendChild(this.container);
        this.initialized = true;
    }

    private createToast(config: ToastConfig): HTMLElement {
        const toast = document.createElement('div');
        toast.className = `toast toast-${config.type || 'info'}`;
        
        const message = document.createElement('div');
        message.className = 'toast-message';
        message.textContent = config.message;
        
        const actions = document.createElement('div');
        actions.className = 'toast-actions';
        
        if (config.retryAction) {
            const retryBtn = document.createElement('button');
            retryBtn.className = 'toast-retry';
            retryBtn.textContent = 'Retry';
            retryBtn.addEventListener('click', async () => {
                retryBtn.disabled = true;
                retryBtn.textContent = 'Retrying...';
                try {
                    await config.retryAction!();
                    this.remove(toast);
                    this.show({ message: 'Success!', type: 'success', duration: 2000 });
                } catch {
                    retryBtn.disabled = false;
                    retryBtn.textContent = 'Retry';
                }
            });
            actions.appendChild(retryBtn);
        }
        
        const closeBtn = document.createElement('button');
        closeBtn.className = 'toast-close';
        closeBtn.textContent = '×';
        closeBtn.addEventListener('click', () => this.remove(toast));
        
        actions.appendChild(closeBtn);
        toast.appendChild(message);
        toast.appendChild(actions);
        
        return toast;
    }

    show(config: ToastConfig): void {
        if (!this.initialized) return;
        if (!this.container) return;
        
        const toast = this.createToast(config);
        this.container.appendChild(toast);
        this.activeToasts.add(toast);
        
        setTimeout(() => {
            toast.classList.add('toast-show');
        }, 10);
        
        if (config.duration && config.duration > 0) {
            setTimeout(() => {
                this.remove(toast);
            }, config.duration);
        }
    }

    remove(toast: HTMLElement): void {
        toast.classList.remove('toast-show');
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
            this.activeToasts.delete(toast);
        }, 300);
    }

    clear(): void {
        this.activeToasts.forEach(toast => this.remove(toast));
    }
}

export const toast = new ToastManager();

export function handleError(error: Error | unknown, context: string, retryAction?: () => Promise<void>) {
    const message = error instanceof Error ? error.message : 'Unknown error occurred';
    console.error(`[${context}] ${message}`, error);
    
    toast.show({
        message: `${context}: ${message}`,
        type: 'error',
        duration: retryAction ? 0 : 5000,
        retryAction
    });
}

export function handleNetworkError(response: Response, context: string, retryAction?: () => Promise<void>) {
    const message = `HTTP ${response.status}: ${response.statusText}`;
    console.error(`[${context}] ${message}`, response);
    
    toast.show({
        message: `${context}: ${message}`,
        type: 'error',
        duration: retryAction ? 0 : 5000,
        retryAction
    });
}