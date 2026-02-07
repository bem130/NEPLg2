import { Shell } from './shell.js';

export interface Span {
    text: string;
    color?: string;
    bg?: string;
}

export class CanvasTerminal {
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
    textarea: HTMLTextAreaElement;
    options: any;
    shell: Shell;
    colors: {
        background: string;
        foreground: string;
        cursor: string;
        input: string;
        orange: string;
        white: string;
        blue: string;
        gray: string;
        green: string;
    };
    history: Span[][];
    promptSpans: Span[];
    currentInput: string;
    cursorIndex: number;
    isComposing: boolean;
    composingText: string;
    fontSize: number;
    fontFamily: string;
    lineHeight: number;
    padding: number;
    charWidth: number;
    charHeight: number;
    scrollTop: number;
    maxScrollTop: number;
    cursorVisible: boolean;
    blinkInterval: any;

    constructor(canvas: HTMLCanvasElement, textarea: HTMLTextAreaElement, domElements: any, options = {}) {
        this.canvas = canvas;
        const ctx = canvas.getContext('2d');
        if (!ctx) throw new Error("Could not get 2D context");
        this.ctx = ctx;
        this.textarea = textarea;
        this.options = options;

        // Dependencies
        this.shell = new Shell(this, null as any);

        // State
        this.colors = {
            background: '#0d1117',
            foreground: '#c9d1d9',
            cursor: '#58a6ff',
            input: '#f0f6fc',
            orange: '#ea5e00',
            white: '#ffffff',
            blue: '#41a6ff',
            gray: '#8b949e',
            green: '#56d364'
        };

        this.history = [];
        this.promptSpans = [];
        this.updatePrompt();

        this.currentInput = "";
        this.cursorIndex = 0;
        this.isComposing = false;
        this.composingText = "";
        this.fontSize = 14;
        this.fontFamily = '"JetBrains Mono", Consolas, monospace';
        this.lineHeight = 1.4;
        this.padding = 10;
        this.charWidth = 0;
        this.charHeight = 0;
        this.scrollTop = 0;
        this.maxScrollTop = 0;
        this.cursorVisible = true;

        this.blinkInterval = setInterval(() => {
            this.cursorVisible = !this.cursorVisible;
            if (!this.isComposing) this.render();
        }, 500);

        this.init();
    }

    init() {
        this.updateMetrics();
        this.bindEvents();
        this.resize();
        this.focus();

        this.print([
            { text: "Welcome to ", color: this.colors.gray },
            { text: "NEPLg2 Playground", color: this.colors.blue },
            { text: " (Web Terminal)", color: this.colors.gray }
        ]);
        this.print([{ text: "Type 'help' for commands.", color: this.colors.green }]);
    }

    updatePrompt() {
        this.promptSpans = [
            { text: "[", color: this.colors.orange },
            { text: "nepl", color: this.colors.white },
            { text: "]", color: this.colors.orange },
            { text: "[", color: this.colors.orange },
            { text: "\uE0A0 main", color: this.colors.blue },
            { text: "]", color: this.colors.orange },
            { text: "[", color: this.colors.orange },
            { text: "/web", color: this.colors.green },
            { text: "]> ", color: this.colors.orange },
        ];
    }

    updateMetrics() {
        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        const metrics = this.ctx.measureText('M');
        this.charWidth = metrics.width;
        this.charHeight = this.fontSize * this.lineHeight;
    }

    bindEvents() {
        this.textarea.addEventListener('input', this.handleInput.bind(this));
        this.textarea.addEventListener('keydown', this.handleKeydown.bind(this));
        this.textarea.addEventListener('compositionstart', this.handleCompositionStart.bind(this));
        this.textarea.addEventListener('compositionupdate', this.handleCompositionUpdate.bind(this));
        this.textarea.addEventListener('compositionend', this.handleCompositionEnd.bind(this));

        this.canvas.addEventListener('mousedown', (e) => {
            this.focus();
            e.preventDefault();
        });

        this.canvas.addEventListener('wheel', (e) => {
            const delta = Math.sign(e.deltaY);
            this.scrollTop += delta;
            this.scrollTop = Math.max(0, Math.min(this.scrollTop, this.maxScrollTop));
            this.render();
            e.preventDefault();
        }, { passive: false });
    }

    resize() {
        const parent = this.canvas.parentElement;
        if (!parent) return;
        const rect = parent.getBoundingClientRect();
        this.canvas.width = rect.width;
        this.canvas.height = rect.height;
        this.updateMetrics();
        this.render();
    }

    resizeEditor() {
        this.resize();
    }

    focus() {
        this.textarea.focus();
    }

    handleInput(e: any) {
        if (this.isComposing) return;
        if (e.data) this.insertText(e.data);
        this.textarea.value = '';
        this.render();
    }

    handleKeydown(e: KeyboardEvent) {
        if (this.isComposing) return;

        switch (e.key) {
            case 'Enter':
                this.execute();
                break;
            case 'Backspace':
                this.deleteBack();
                break;
            case 'ArrowLeft':
                if (this.cursorIndex > 0) this.cursorIndex--;
                break;
            case 'ArrowRight':
                if (this.cursorIndex < this.currentInput.length) this.cursorIndex++;
                break;
            case 'ArrowUp':
                this.navigateHistory(-1);
                e.preventDefault();
                break;
            case 'ArrowDown':
                this.navigateHistory(1);
                e.preventDefault();
                break;
            case 'c':
                if (e.ctrlKey) {
                    const cancelledState = [
                        ...this.promptSpans,
                        { text: this.currentInput + "^C", color: this.colors.input }
                    ];
                    this.history.push(cancelledState);
                    this.currentInput = "";
                    this.cursorIndex = 0;
                    this.updateScrollTopToBottom();
                    this.render();
                    e.preventDefault();
                }
                break;
        }

        this.restartBlink();
        this.updateInputPosition();
        this.render();
    }

    navigateHistory(direction: number) {
        if (!this.shell) return;
        const history = this.shell.history;
        if (history.length === 0) return;

        let newIndex = this.shell.historyIndex + direction;
        if (newIndex < 0) newIndex = 0;
        if (newIndex > history.length) newIndex = history.length;

        if (newIndex === history.length) {
            this.currentInput = "";
        } else {
            this.currentInput = history[newIndex];
        }
        this.shell.historyIndex = newIndex;
        this.cursorIndex = this.currentInput.length;
    }

    handleCompositionStart() {
        this.isComposing = true;
    }

    handleCompositionUpdate(e: any) {
        this.composingText = e.data;
        this.render();
        this.updateInputPosition();
    }

    handleCompositionEnd(e: any) {
        this.isComposing = false;
        this.insertText(e.data);
        this.composingText = "";
        this.textarea.value = "";
        this.render();
    }

    insertText(text: string) {
        const pre = this.currentInput.slice(0, this.cursorIndex);
        const post = this.currentInput.slice(this.cursorIndex);
        this.currentInput = pre + text + post;
        this.cursorIndex += text.length;
    }

    deleteBack() {
        if (this.cursorIndex > 0) {
            const pre = this.currentInput.slice(0, this.cursorIndex - 1);
            const post = this.currentInput.slice(this.cursorIndex);
            this.currentInput = pre + post;
            this.cursorIndex--;
        }
    }

    async execute() {
        const cmd = this.currentInput;
        const cmdLine = [
            ...this.promptSpans,
            { text: cmd, color: this.colors.input }
        ];
        this.history.push(cmdLine);

        this.currentInput = "";
        this.cursorIndex = 0;
        this.isComposing = false;

        if (cmd.trim()) {
            await this.shell.executeLine(cmd);
        }

        this.updateScrollTopToBottom();
        this.render();
    }

    print(content: string | Span[]) {
        if (typeof content === 'string') {
            const lines = content.split('\n');
            for (const line of lines) {
                this.history.push([{ text: line, color: this.colors.foreground }]);
            }
        } else if (Array.isArray(content)) {
            this.history.push(content);
        }
        this.updateScrollTopToBottom();
        this.render();
    }

    updateScrollTopToBottom() {
        const maxVisibleLines = Math.floor((this.canvas.height - this.padding * 2) / this.charHeight);
        const totalLines = this.history.length + 1;
        this.maxScrollTop = Math.max(0, totalLines - maxVisibleLines);
        this.scrollTop = this.maxScrollTop;
    }

    printError(text: string) {
        this.print([{ text: text, color: '#ff7b72' }]);
    }

    clear() {
        this.history = [];
        this.scrollTop = 0;
        this.maxScrollTop = 0;
        this.render();
    }

    copyAll() {
        const text = this.history.map(line => line.map(span => span.text).join('')).join('\n');
        navigator.clipboard.writeText(text).then(() => {
            this.print([{ text: "Copied entire buffer to clipboard.", color: this.colors.green }]);
        });
    }

    render() {
        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        this.ctx.textBaseline = 'top';

        const maxVisibleLines = Math.floor((this.canvas.height - this.padding * 2) / this.charHeight);
        const totalLines = this.history.length + 1;
        this.maxScrollTop = Math.max(0, totalLines - maxVisibleLines);

        let startLine = this.scrollTop;
        let y = this.padding;

        const drawLine = (spans: Span[], startX: number) => {
            let x = startX;
            for (const span of spans) {
                this.ctx.fillStyle = span.color || this.colors.foreground;
                this.ctx.fillText(span.text, x, y);
                x += this.ctx.measureText(span.text).width;
            }
            return x;
        };

        for (let i = startLine; i < this.history.length; i++) {
            if (y + this.charHeight > this.canvas.height - this.padding) break;
            drawLine(this.history[i], this.padding);
            y += this.charHeight;
        }

        if (this.history.length >= startLine && y + this.charHeight <= this.canvas.height - this.padding) {
            let x = drawLine(this.promptSpans, this.padding);
            const preInput = this.currentInput.slice(0, this.cursorIndex);
            this.ctx.fillStyle = this.colors.input;
            this.ctx.fillText(preInput, x, y);
            let inputX = x + this.ctx.measureText(preInput).width;

            if (this.isComposing) {
                this.ctx.fillStyle = this.colors.foreground;
                this.ctx.fillText(this.composingText, inputX, y);
                const compWidth = this.ctx.measureText(this.composingText).width;
                this.ctx.beginPath();
                this.ctx.moveTo(inputX, y + this.charHeight - 2);
                this.ctx.lineTo(inputX + compWidth, y + this.charHeight - 2);
                this.ctx.strokeStyle = this.colors.cursor;
                this.ctx.stroke();
                inputX += compWidth;
            }

            if (this.cursorVisible && !this.isComposing) {
                this.ctx.fillStyle = this.colors.cursor;
                this.ctx.fillRect(inputX, y, 2, this.charHeight);
            }

            const postInput = this.currentInput.slice(this.cursorIndex);
            this.ctx.fillStyle = this.colors.input;
            this.ctx.fillText(postInput, inputX, y);
        }
    }

    restartBlink() {
        this.cursorVisible = true;
        clearInterval(this.blinkInterval);
        this.blinkInterval = setInterval(() => {
            this.cursorVisible = !this.cursorVisible;
            if (!this.isComposing) this.render();
        }, 500);
    }

    updateInputPosition() {
        let promptWidth = 0;
        this.ctx.font = `${this.fontSize}px ${this.fontFamily}`;
        for (const span of this.promptSpans) {
            promptWidth += this.ctx.measureText(span.text).width;
        }

        const preText = this.currentInput.slice(0, this.cursorIndex) + (this.isComposing ? this.composingText : "");
        const inputWidth = this.ctx.measureText(preText).width;
        const x = this.padding + promptWidth + inputWidth;

        const maxVisibleLines = Math.floor((this.canvas.height - this.padding * 2) / this.charHeight);
        let visualRow = this.history.length - this.scrollTop;
        if (visualRow >= maxVisibleLines) visualRow = maxVisibleLines - 1;

        const y = this.padding + visualRow * this.charHeight;

        this.textarea.style.left = `${this.canvas.offsetLeft + x}px`;
        this.textarea.style.top = `${this.canvas.offsetTop + y}px`;
        this.textarea.style.height = `${this.charHeight}px`;
    }
}
