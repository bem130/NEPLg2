import { WASI } from '../runtime/wasi.js';

export class Shell {
    constructor(terminal, vfs) {
        this.terminal = terminal;
        this.vfs = vfs; // Virtual File System
        this.env = new Map();
        this.history = []; // String history for Arrow UP/DOWN
        this.historyIndex = 0;
        this.editor = null; // To be injected
    }

    async executeLine(line) {
        if (!line.trim()) return;
        this.history.push(line);
        this.historyIndex = this.history.length;

        const segments = line.split('|').map(s => s.trim());
        let inputData = null;

        for (let i = 0; i < segments.length; i++) {
            const segment = segments[i];
            const args = this.parseArgs(segment);
            if (args.length === 0) continue;

            const cmd = args[0];
            const cmdArgs = args.slice(1);

            try {
                const isLast = (i === segments.length - 1);
                const output = await this.runCommand(cmd, cmdArgs, inputData);

                if (isLast) {
                    if (output) this.terminal.print(output);
                } else {
                    inputData = output;
                }
            } catch (e) {
                this.terminal.printError(`Error: ${e.message}`);
                break;
            }
        }
    }

    parseArgs(segment) {
        const args = [];
        let current = '';
        let inQuote = false;

        for (let i = 0; i < segment.length; i++) {
            const char = segment[i];
            if (char === '"') {
                inQuote = !inQuote;
            } else if (char === ' ' && !inQuote) {
                if (current) {
                    args.push(current);
                    current = '';
                }
            } else {
                current += char;
            }
        }
        if (current) args.push(current);
        return args;
    }

    async runCommand(cmd, args, stdin) {
        switch (cmd) {
            case 'echo':
                if (args.length > 0) return args.join(' ');
                return stdin || "";

            case 'clear':
            case 'cls':
                this.terminal.clear();
                return null;

            case 'help':
                return [
                    "Available commands:",
                    "  help      Show this help message",
                    "  clear     Clear the terminal screen",
                    "  echo      Print arguments to stdout",
                    "  ls        List files in the virtual file system",
                    "  tree      Recursive directory listing",
                    "  cat       Display file content",
                    "  copy      Copy terminal buffer to clipboard",
                    "  neplg2    NEPLg2 Compiler & Toolchain",
                    "    run     Compile and run the current editor content",
                    "    build   Compile the editor or a file",
                    "  run       Alias for 'neplg2 run'",
                    "  compile   Alias for 'neplg2 build --emit wat'",
                    "  test      Alias for 'neplg2 run stdlib/test.nepl' (if applicable)",
                    "  wasmi     Run a .wasm file"
                ].join('\n');

            case 'run':
                return await this.cmdNeplg2(['run'], stdin);

            case 'build':
                return await this.cmdNeplg2(['build'], stdin);

            case 'compile':
                return await this.cmdNeplg2(['build', '--emit', 'wat'], stdin);

            case 'test':
                return await this.cmdNeplg2(['run', 'stdlib/std/test.nepl'], stdin);

            case 'neplg2':
                return await this.cmdNeplg2(args, stdin);

            case 'wasmi':
                return await this.cmdWasmi(args, stdin);

            case 'ls':
                const lsPath = args[0] || '/';
                try {
                    if (this.vfs.isDir(lsPath)) {
                        const entries = this.vfs.listDir(lsPath);
                        const mapped = entries.map(entry => {
                            const fullPath = (lsPath.endsWith('/') ? lsPath : lsPath + '/') + entry;
                            return this.vfs.isDir(fullPath) ? entry + '/' : entry;
                        });
                        return mapped.join('\n');
                    }
                    return this.vfs.exists(lsPath) ? args[0] : `ls: no such file or directory: ${lsPath}`;
                } catch (e) {
                    return `ls: ${e.message}`;
                }

            case 'tree':
                const treePath = args[0] || '/';
                try {
                    return this.renderTree(treePath);
                } catch (e) {
                    return `tree: ${e.message}`;
                }

            case 'cat':
                if (args.length === 0) return "cat: missing operand";
                const catPath = args[0];
                try {
                    if (this.vfs.isDir(catPath)) return `cat: ${catPath}: Is a directory`;
                    const content = this.vfs.readFile(catPath);
                    if (typeof content === 'string') return content;
                    return "[Binary content]";
                } catch (e) {
                    return `cat: ${catPath}: No such file`;
                }

            case 'copy':
                this.terminal.copyAll();
                return null;

            case 'vfs_debug':
                console.log(this.vfs);
                return "Dumped to console";

            default:
                throw new Error(`Unknown command: ${cmd}`);
        }
    }

    async cmdNeplg2(args, stdin) {
        const parsed = this.parseFlags(args);

        if (args.includes('run') || args.includes('build')) {
            this.terminal.print("Compiling...");

            let inputFile = parsed.flags['-i'] || parsed.flags['--input'];
            if (!inputFile) {
                const lastPos = parsed.positional[parsed.positional.length - 1];
                if (lastPos && lastPos !== 'run' && lastPos !== 'build') {
                    inputFile = lastPos;
                }
            }

            let source = "";
            let inputPath = "editor";

            if (inputFile) {
                if (!this.vfs.exists(inputFile)) return `Error: File not found '${inputFile}'`;
                source = this.vfs.readFile(inputFile);
                inputPath = inputFile;
            } else {
                if (this.editor) {
                    if (typeof this.editor.getText === 'function') {
                        source = this.editor.getText();
                    } else if (this.editor.text !== undefined) {
                        source = this.editor.text;
                    } else {
                        return "Error: Could not retrieve text from editor";
                    }
                    this.terminal.print("(Using editor content)");
                } else {
                    return "Error: Editor not connected";
                }
            }
            this.terminal.print(`Source: ${inputPath}`);

            if (!window.wasmBindings) return "Error: Compiler (WASM) not loaded yet.";

            try {
                if (parsed.flags['--emit'] && parsed.flags['--emit'].includes('wat')) {
                    const wat = window.wasmBindings.compile_to_wat(source);
                    this.vfs.writeFile('out.wat', wat);
                    this.terminal.print("Generated out.wat");
                }

                const wasm = window.wasmBindings.compile_source(source);
                const outFile = parsed.flags['-o'] || 'out.wasm';
                this.vfs.writeFile(outFile, wasm);
                this.terminal.print(`Compilation finished. Output to ${outFile}`);

                if (args.includes('run')) {
                    return await this.cmdWasmi([outFile], stdin);
                }
                return "Build complete.";
            } catch (e) {
                return `Compilation Failed: ${e}`;
            }
        }
        return "Unknown neplg2 command.";
    }

    parseFlags(args) {
        const flags = {};
        const positional = [];
        for (let i = 0; i < args.length; i++) {
            if (args[i].startsWith('-')) {
                if (i + 1 < args.length && !args[i + 1].startsWith('-')) {
                    flags[args[i]] = args[i + 1];
                    i++;
                } else {
                    flags[args[i]] = true;
                }
            } else {
                positional.push(args[i]);
            }
        }
        return { flags, positional };
    }

    async cmdWasmi(args, stdin) {
        if (args.length === 0) return "wasmi: missing file";
        const filename = args[0];
        if (!this.vfs.exists(filename)) return `wasmi: file not found: ${filename}`;

        const bin = this.vfs.readFile(filename);
        if (!(bin instanceof Uint8Array)) return "wasmi: invalid binary format";

        this.terminal.print(`Executing ${filename} ...`);

        try {
            const wasi = new WASI(args, this.env, this.vfs, this.terminal);
            const { instance } = await WebAssembly.instantiate(bin, wasi.imports);
            wasi.setMemory(instance.exports.memory);

            if (instance.exports._start) {
                instance.exports._start();
            } else if (instance.exports.main) {
                const res = instance.exports.main();
                return `Exited with ${res}`;
            } else {
                return "wasmi: no entry point found";
            }
        } catch (e) {
            if (e.message && e.message.includes("Exited with code")) return e.message;
            return `wasmi error: ${e}`;
        }
        return "Program exited.";
    }

    renderTree(rootPath) {
        if (!rootPath.startsWith('/')) rootPath = '/' + rootPath;
        const results = [];
        results.push(rootPath);

        const build = (path, prefix) => {
            const entries = this.vfs.listDir(path);
            for (let i = 0; i < entries.length; i++) {
                const entry = entries[i];
                const isLast = i === entries.length - 1;
                const fullPath = (path.endsWith('/') ? path : path + '/') + entry;
                const isDir = this.vfs.isDir(fullPath);

                results.push(`${prefix}${isLast ? '└── ' : '├── '}${(isDir ? entry + '/' : entry)}`);

                if (isDir) {
                    build(fullPath, prefix + (isLast ? '    ' : '│   '));
                }
            }
        };

        build(rootPath, '');
        return results.join('\n');
    }
}
