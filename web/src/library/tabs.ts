export interface Tab {
    path: string;
    content: string;
}

export class TabManager {
    tabs: Tab[] = [];
    activeTabIndex: number = -1;
    container: HTMLElement;
    editor: any;
    vfs: any;

    constructor(container: HTMLElement, editor: any, vfs: any) {
        this.container = container;
        this.editor = editor;
        this.vfs = vfs;
    }

    openFile(path: string) {
        // Save current tab first
        this.saveCurrentTab();

        let index = this.tabs.findIndex(t => t.path === path);
        if (index === -1) {
            const content = this.vfs.readFile(path);
            this.tabs.push({ path, content: typeof content === 'string' ? content : "Binary file..." });
            index = this.tabs.length - 1;
        }

        this.setActiveTab(index);
    }

    saveCurrentTab() {
        if (this.activeTabIndex >= 0) {
            const currentTab = this.tabs[this.activeTabIndex];
            currentTab.content = typeof this.editor.getText === 'function' ? this.editor.getText() : this.editor.text;
            this.vfs.writeFile(currentTab.path, currentTab.content);
        }
    }

    setActiveTab(index: number) {
        this.activeTabIndex = index;
        const tab = this.tabs[index];
        this.editor.setText(tab.content);
        // Explicitly set the path on the editor if possible
        if (this.editor) {
            (this.editor as any).path = tab.path;
        }
        this.render();
    }

    closeTab(index: number, e?: Event) {
        if (e) e.stopPropagation();
        this.tabs.splice(index, 1);
        if (this.activeTabIndex === index) {
            this.activeTabIndex = this.tabs.length > 0 ? 0 : -1;
            if (this.activeTabIndex >= 0) {
                this.setActiveTab(this.activeTabIndex);
            } else {
                this.editor.setText("");
                if (this.editor) (this.editor as any).path = null;
            }
        } else if (this.activeTabIndex > index) {
            this.activeTabIndex--;
        }
        this.render();
    }

    render() {
        this.container.innerHTML = "";
        this.tabs.forEach((tab, i) => {
            const el = document.createElement('div');
            el.className = `tab ${i === this.activeTabIndex ? 'active' : ''}`;

            const title = document.createElement('span');
            title.className = 'tab-title';
            title.textContent = tab.path.split('/').pop() || tab.path;

            const close = document.createElement('span');
            close.className = 'tab-close';
            close.textContent = '×';
            close.onclick = (e) => this.closeTab(i, e);

            el.appendChild(title);
            el.appendChild(close);
            el.onclick = () => this.setActiveTab(i);

            this.container.appendChild(el);
        });
    }

    get activeTab(): Tab | null {
        return this.activeTabIndex >= 0 ? this.tabs[this.activeTabIndex] : null;
    }
}
