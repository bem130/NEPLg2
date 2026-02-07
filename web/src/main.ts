import { CanvasTerminal } from './terminal/terminal.js';
import { VFS } from './runtime/vfs.js';
import { TabManager } from './library/tabs.js';
import { FileExplorer } from './library/explorer.js';

declare const NEPLg2LanguageProvider: any;
declare const CanvasEditorLibrary: any;

console.log("[Playground] main.js loaded (TS-MIGRATION)");
let start_flag = false;

window.addEventListener("TrunkApplicationStarted", start_app);
window.setTimeout(start_app, 1000);

function start_app() {
    if (start_flag) return;
    start_flag = true;

    // --- Core Dependencies ---
    console.log("[Playground] Initializing VFS...");
    const vfs = new VFS();

    let wasm: any;
    try {
        wasm = (window as any).wasmBindings
    }
    catch (e) {
        console.error("[Playground] WASM bindings not found, retrying in 1 second...", e);
        start_flag = false; // Allow retry
        window.setTimeout(start_app, 1000);
        return;
    }
    console.log("[Playground] Trunk application started. Initializing...");

    if (wasm && wasm.initSync) {
        try {
            wasm.initSync();
            console.log("[Playground] WASM initSync complete.");

            // Mount stdlib into VFS
            if (wasm.get_stdlib_files) {
                const stdlibFiles = wasm.get_stdlib_files();
                if (stdlibFiles && Array.isArray(stdlibFiles)) {
                    for (const [path, content] of stdlibFiles) {
                        vfs.writeFile('/stdlib/' + path, content);
                    }
                }
            }

            // Mount examples into VFS
            if (wasm.get_example_files) {
                const exampleFiles = wasm.get_example_files();
                if (exampleFiles && Array.isArray(exampleFiles)) {
                    for (const [path, content] of exampleFiles) {
                        vfs.writeFile('/examples/' + path, content);
                    }
                }
            }

            // Load README
            if (wasm.get_readme) {
                const readme = wasm.get_readme();
                vfs.writeFile('/README', readme);
            }
        } catch (e) {
            console.error("[Playground] WASM initSync failed:", e);
        }
    }

    // --- DOM Elements ---
    const editorCanvas = document.getElementById('editor-canvas') as HTMLCanvasElement;
    const editorTextarea = document.getElementById('editor-hidden-input') as HTMLTextAreaElement;
    const completionList = document.getElementById('completion-list') as HTMLElement;
    const generalPopup = document.getElementById('general-popup') as HTMLElement;
    const terminalCanvas = document.getElementById('terminal-canvas') as HTMLCanvasElement;
    const terminalTextarea = document.getElementById('terminal-hidden-input') as HTMLTextAreaElement;
    const fontSizeSelect = document.getElementById('font-size-select') as HTMLSelectElement;

    const explorerContent = document.getElementById('explorer-content') as HTMLElement;
    const tabsContainer = document.getElementById('tabs-container') as HTMLElement;
    const refreshExplorerBtn = document.getElementById('refresh-explorer') as HTMLElement;

    // --- Editor Setup ---
    const neplProvider = new NEPLg2LanguageProvider();
    const { editor } = CanvasEditorLibrary.createCanvasEditor({
        canvas: editorCanvas,
        textarea: editorTextarea,
        popup: generalPopup,
        completionList: completionList,
        languageProviders: {
            nepl: neplProvider
        },
        initialLanguage: 'nepl'
    });

    // --- Tab & Explorer Setup ---
    const tabManager = new TabManager(tabsContainer, editor, vfs);
    const fileExplorer = new FileExplorer(explorerContent, vfs, (path) => {
        tabManager.openFile(path);
    });

    // --- Terminal Setup ---
    const terminal = new CanvasTerminal(terminalCanvas, terminalTextarea, null, { vfs });

    // Inject dependencies into shell
    if (terminal.shell) {
        terminal.shell.editor = editor;
        terminal.shell.vfs = vfs;
        (terminal.shell as any).tabManager = tabManager;
    }

    // --- Resizer Logic ---
    const setupResizer = (resizerId: string, leftPaneId: string, isHorizontal: boolean) => {
        const resizer = document.getElementById(resizerId);
        const leftPane = document.getElementById(leftPaneId);
        if (!resizer || !leftPane) return;

        let isResizing = false;

        resizer.addEventListener('mousedown', (e) => {
            isResizing = true;
            document.body.style.cursor = isHorizontal ? 'col-resize' : 'row-resize';
            resizer.classList.add('dragging');
        });

        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;
            if (isHorizontal) {
                const width = e.clientX - leftPane.getBoundingClientRect().left;
                leftPane.style.width = width + 'px';
            }
            editor.resizeEditor();
            terminal.resizeEditor();
        });

        document.addEventListener('mouseup', () => {
            isResizing = false;
            document.body.style.cursor = 'default';
            resizer.classList.remove('dragging');
        });
    };

    setupResizer('explorer-resizer', 'explorer-pane', true);
    setupResizer('workspace-resizer', 'editor-pane', true);

    // --- Simple Commands ---
    function executeCommand(cmd: string) {
        tabManager.saveCurrentTab(); // Sync before execution
        terminal.currentInput = cmd;
        terminal.execute();
    }

    function updateFontSize() {
        const size = parseInt(fontSizeSelect.value);
        editor.setFontSize(size);
        terminal.setFontSize(size);
    }

    // --- Event Listeners ---
    fontSizeSelect.addEventListener('change', updateFontSize);
    refreshExplorerBtn.addEventListener('click', () => fileExplorer.refresh());

    window.addEventListener('resize', () => {
        editor.resizeEditor();
        terminal.resizeEditor();
    });

    // --- Initialization ---
    fileExplorer.render();
    tabManager.openFile('/examples/rpn.nepl');

    // Make globally available
    (window as any).editor = editor;
    (window as any).terminal = terminal;
    (window as any).executeCommand = executeCommand;
    (window as any).tabManager = tabManager;

    setTimeout(() => {
        editor.resizeEditor();
        terminal.resizeEditor();
        editor.focus();
    }, 100);
}
