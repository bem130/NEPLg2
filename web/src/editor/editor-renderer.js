export class EditorRenderer {
    constructor(editor) {
        this.editor = editor;
        this.canvas = editor.canvas;
        this.ctx = editor.ctx;
        this.colors = editor.colors;
        this.lastBlinkTime = 0;
    }

    recalculateLinePositions() {
        this.editor.lineYPositions = [];
        let currentY = this.editor.geom.padding;
        for (let i = 0; i < this.editor.lines.length; i++) {
            this.editor.lineYPositions[i] = currentY;
            const range = this.editor.foldingRanges.find(r => r.startLine === i);
            if (range && this.editor.foldedLines.has(i)) {
                // If folded, skip lines
                // Simplified: just update index, real logic needs to skip iterations or mark hidden
                // For this port, we are not fully implementing text folding logic yet, just placeholders
                // But we must honor the structure for rendering
            }
            currentY += this.editor.geom.lineHeight;
        }
    }

    renderLoop(timestamp) {
        this.updateCursorBlink(timestamp);
        this.render();
        this.editor.updateTextareaPosition();
        requestAnimationFrame(this.renderLoop.bind(this));
    }

    updateCursorBlink(timestamp) {
        if (!this.editor.isFocused) return;
        if (timestamp - this.lastBlinkTime > this.editor.blinkInterval) {
            this.editor.cursorBlinkState = !this.editor.cursorBlinkState;
            this.lastBlinkTime = timestamp;
        }
    }

    render() {
        this.recalculateLinePositions();
        const dpr = window.devicePixelRatio || 1;
        // Use parent rect
        const rect = this.canvas.parentElement.getBoundingClientRect();

        this.ctx.fillStyle = this.colors.background;
        this.ctx.fillRect(0, 0, this.canvas.width / dpr, this.canvas.height / dpr);

        this.ctx.fillStyle = this.colors.gutterBackground;
        this.ctx.fillRect(0, 0, this.editor.geom.gutterWidth, this.canvas.height / dpr);

        this.ctx.save();
        this.ctx.translate(-this.editor.scrollX, -this.editor.scrollY);
        const selection = this.editor.getSelectionRange();
        const cursorPosition = this.editor.utils.getPosFromIndex(this.editor.cursor, this.editor.lines);

        for (let i = 0; i < this.editor.lines.length; i++) {
            const y = this.editor.lineYPositions[i];
            // Culling
            if (y + this.editor.geom.lineHeight < this.editor.scrollY || y > this.editor.scrollY + (this.canvas.height / dpr)) {
                continue;
            }

            const line = this.editor.lines[i];
            const textY = y + this.editor.geom.lineHeight / 2;

            // Cursor Line Highlight
            if (this.editor.isFocused && !this.editor.hasSelection() && cursorPosition.row === i) {
                this.ctx.strokeStyle = this.colors.cursorLineBorder;
                this.ctx.lineWidth = 1;
                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.geom.gutterWidth + this.editor.scrollX, y);
                this.ctx.lineTo(this.editor.scrollX + (this.canvas.width / dpr), y);
                this.ctx.stroke();

                this.ctx.beginPath();
                this.ctx.moveTo(this.editor.geom.gutterWidth + this.editor.scrollX, y + this.editor.geom.lineHeight);
                this.ctx.lineTo(this.editor.scrollX + (this.canvas.width / dpr), y + this.editor.geom.lineHeight);
                this.ctx.stroke();
            }

            // Line Numbers
            this.ctx.textAlign = 'right';
            this.ctx.fillStyle = (this.editor.isFocused && cursorPosition.row === i) ? this.colors.lineNumberActive : this.colors.lineNumber;
            this.ctx.fillText(String(i + 1), this.editor.geom.gutterWidth - this.editor.geom.padding, textY);
            this.ctx.textAlign = 'left';

            const lineStartX = this.editor.geom.padding + this.editor.geom.gutterWidth;

            // Draw Content Logic
            // We use a helper similar to editorsample's drawLineContent
            const drawLineContent = (text, startX, textY, lineStartIndexOffset = 0) => {
                let currentX = startX;
                let isLeading = true;
                let spaceCountInIndent = 0;

                // Find index of last non-space char to determine trailing spaces
                const match = line.match(/\s*$/);
                const lastNonSpaceIndex = (match && match.index !== undefined) ? match.index : line.length;
                // If line is empty string, lastNonSpaceIndex is 0, so all spaces are trailing? yes.

                for (let j = 0; j < text.length; j++) {
                    const char = text[j];
                    const charWidth = this.editor.utils.getCharWidth(char);
                    const charIndex = this.editor.utils.getIndexFromPos(i, 0, this.editor.lines) + lineStartIndexOffset + j;

                    // Determine if trailing
                    // We need start index of line
                    const lineStartGlobal = this.editor.utils.getIndexFromPos(i, 0, this.editor.lines);
                    const indexInLine = charIndex - lineStartGlobal;
                    const isTrailing = indexInLine >= lastNonSpaceIndex;

                    // Highlight Occurrences (TODO: Port functionality)
                    // ...

                    // Indent Highlighting
                    if (this.editor.langConfig.highlightIndent && isLeading) {
                        if (char === ' ') {
                            spaceCountInIndent++;
                            this.ctx.fillStyle = this.colors.indentation[(Math.floor((spaceCountInIndent - 1) / 4)) % 2];
                            this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight);
                        }
                        else if (char === '\t') {
                            this.ctx.fillStyle = this.colors.tab;
                            this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight);
                        }
                        else { isLeading = false; }
                    } else { isLeading = false; }

                    // Trailing Space Highlighting
                    if (isTrailing && (char === ' ' || char === '\t' || char === '　')) {
                        this.ctx.fillStyle = this.colors.trailingSpace;
                        this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight);
                    }

                    // Selection
                    if (charIndex >= selection.start && charIndex < selection.end) {
                        this.ctx.fillStyle = this.colors.selection;
                        this.ctx.fillRect(currentX, y, charWidth, this.editor.geom.lineHeight);
                    }

                    // Syntax Coloring
                    const token = this.editor.tokens.find(t => charIndex >= t.startIndex && charIndex < t.endIndex);
                    this.ctx.fillStyle = token ? (this.colors.tokenColors[token.type] || this.colors.tokenColors.default) : this.colors.text;

                    // Specific character rendering
                    if (this.editor.langConfig.highlightWhitespace) {
                        if (char === ' ') {
                            // Optionally draw dot
                        }
                    }

                    this.ctx.fillText(char, currentX, textY);

                    currentX += charWidth;
                }
                return currentX;
            };

            if (this.editor.isFocused && this.editor.isComposing && cursorPosition.row === i) {
                // Split line for composition
                const lineBefore = line.substring(0, cursorPosition.col);
                const lineAfter = line.substring(cursorPosition.col);

                let currentX = drawLineContent(lineBefore, lineStartX, textY);
                const imeStartX = currentX;

                this.ctx.fillStyle = this.colors.text;
                let imeCurrentX = currentX;
                for (const char of this.editor.compositionText) {
                    this.ctx.fillText(char, imeCurrentX, textY);
                    imeCurrentX += this.editor.utils.getCharWidth(char);
                }

                const compositionWidth = this.editor.utils.measureText(this.editor.compositionText);
                this.ctx.strokeStyle = this.colors.imeUnderline;
                this.ctx.lineWidth = 1 / dpr;
                this.ctx.beginPath();
                this.ctx.moveTo(imeStartX, y + this.editor.geom.lineHeight - 2);
                this.ctx.lineTo(imeStartX + compositionWidth, y + this.editor.geom.lineHeight - 2);
                this.ctx.stroke();

                currentX += compositionWidth;

                // Note: Indentation highlight logic usually breaks if split like this because 'isLeading' resets.
                // But typically you don't compose at the start of line often enough for it to matter heavily for this demo.
                drawLineContent(lineAfter, currentX, textY, cursorPosition.col);
            } else {
                drawLineContent(line, lineStartX, textY);
            }

            // Diagnostics Mock
            // ...
        }

        // Cursor Drawing
        if (this.editor.isFocused && !this.editor.isComposing) {
            const cursorPos = this.editor.utils.getCursorCoords(this.editor.cursor, this.editor.lines, this.editor.lineYPositions);
            if (cursorPos.y > -1) {
                if (this.editor.isOverwriteMode) {
                    // ...
                } else if (this.editor.cursorBlinkState && !this.editor.hasSelection()) {
                    this.ctx.fillStyle = this.colors.cursor;
                    this.ctx.fillRect(cursorPos.x, cursorPos.y, 2, this.editor.geom.lineHeight);
                }
            }
        }

        this.ctx.restore();
    }
}
