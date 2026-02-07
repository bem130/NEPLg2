import { CanvasTerminal } from './terminal/terminal.js';
import { VFS } from './runtime/vfs.js';

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

    console.log("[Playground] WASM bindings:", wasm);
    if (wasm && wasm.initSync) {
        try {
            wasm.initSync();
            console.log("[Playground] WASM initSync complete.");

            // Mount stdlib into VFS
            if (wasm.get_stdlib_files) {
                const stdlibFiles = wasm.get_stdlib_files();
                if (stdlibFiles && Array.isArray(stdlibFiles)) {
                    console.log(`[Playground] Mounting ${stdlibFiles.length} stdlib files...`);
                    for (const [path, content] of stdlibFiles) {
                        vfs.writeFile('/stdlib/' + path, content);
                    }
                }
            }

            // Mount examples into VFS
            if (wasm.get_example_files) {
                const exampleFiles = wasm.get_example_files();
                if (exampleFiles && Array.isArray(exampleFiles)) {
                    console.log(`[Playground] Mounting ${exampleFiles.length} example files into /examples/`);
                    for (const [path, content] of exampleFiles) {
                        vfs.writeFile('/examples/' + path, content);
                    }
                }
            }

            // Load README
            if (wasm.get_readme) {
                const readme = wasm.get_readme();
                vfs.writeFile('/README', readme);
                console.log("[Playground] README mounted to VFS.");
            }
        } catch (e) {
            console.error("[Playground] WASM initSync failed:", e);
        }
    }

    // --- DOM Elements ---
    const editorCanvas = document.getElementById('editor-canvas') as HTMLCanvasElement;
    const editorTextarea = document.getElementById('editor-hidden-input') as HTMLTextAreaElement;
    const editorStatus = document.getElementById('editor-status');
    const completionList = document.getElementById('completion-list') as HTMLElement;
    const generalPopup = document.getElementById('general-popup') as HTMLElement;
    const terminalCanvas = document.getElementById('terminal-canvas') as HTMLCanvasElement;
    const terminalTextarea = document.getElementById('terminal-hidden-input') as HTMLTextAreaElement;
    const exampleSelect = document.getElementById('example-select') as HTMLSelectElement;

    // --- Editor Setup ---
    console.log("[Playground] Setting up CanvasEditor...");
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

    // --- Terminal Setup ---
    console.log("[Playground] Setting up CanvasTerminal...");
    const terminal = new CanvasTerminal(terminalCanvas, terminalTextarea, null, {});

    // Inject dependencies into shell
    if (terminal.shell) {
        terminal.shell.editor = editor;
        terminal.shell.vfs = vfs;
        console.log("[Playground] Shell dependencies injected.");
    }

    // --- Simple Commands for Buttons ---
    function executeCommand(cmd: string) {
        console.log(`[Playground] Executing command: ${cmd}`);
        terminal.currentInput = cmd;
        terminal.execute();
    }

    // --- Example Loading Logic ---
    async function loadExamples() {
        console.log("[Playground] Scanning VFS for examples...");
        const examples = vfs.listDir('/examples');
        console.log("[Playground] Examples listed from VFS:", examples);

        exampleSelect.innerHTML = '<option value="" disabled selected>Select an example...</option>';

        for (const file of examples) {
            const option = document.createElement('option');
            option.value = file;
            option.textContent = file;
            exampleSelect.appendChild(option);
        }

        if (examples.includes('rpn.nepl')) {
            await loadExample('rpn.nepl');
        } else if (examples.length > 0) {
            await loadExample(examples[0]);
        }
    }

    async function loadExample(filename: string) {
        console.log(`[Playground] Loading example from VFS: ${filename}`);
        try {
            const path = '/examples/' + filename;
            if (!vfs.exists(path)) {
                console.error(`[Playground] File not found in VFS: ${path}`);
                return;
            }
            const text = vfs.readFile(path) as string;
            editor.setText(text);
            if (editorStatus) editorStatus.textContent = path.substring(1); // "examples/..."
            terminal.print([
                { text: "Loaded ", color: "#56d364" },
                { text: filename, color: "#58a6ff" }
            ]);
            exampleSelect.value = filename;
        } catch (error) {
            console.error(`[Playground] Error loading example ${filename}:`, error);
            terminal.printError(`Error loading ${filename}: ${error}`);
        }
    }

    async function loadSelectedExample() {
        const selectedFile = exampleSelect.value;
        if (!selectedFile) return;
        await loadExample(selectedFile);
    }

    // --- Event Listeners ---
    exampleSelect.addEventListener('change', loadSelectedExample);

    window.addEventListener('resize', () => {
        editor.resizeEditor();
        terminal.resizeEditor();
    });

    // --- Initialization ---
    loadExamples();

    // Make globally available
    (window as any).editor = editor;
    (window as any).terminal = terminal;
    (window as any).executeCommand = executeCommand;

    // Initial resize and focus
    setTimeout(() => {
        editor.resizeEditor();
        terminal.resizeEditor();
        editor.focus();
    }, 100);
}