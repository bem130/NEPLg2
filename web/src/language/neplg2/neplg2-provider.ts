// @ts-nocheck
class NEPLg2LanguageProvider {
    constructor() {
        this.updateCallback = () => {};
        this.text = '';
        this.lex = null;
        this.parse = null;
        this.resolve = null;
        this.semantics = null;
        this.keywordCompletions = [
            'fn', 'let', 'mut', 'set', 'if', 'while', 'cond', 'then', 'else', 'do',
            'block', 'return', 'break', 'match', 'trait', 'impl', 'for', 'enum', 'struct',
            '#entry', '#target', '#indent', '#import', '#use'
        ];
    }

    onUpdate(callback) {
        this.updateCallback = callback || (() => {});
    }

    updateText(text) {
        this.text = text || '';
        this._analyzeAndPublish();
    }

    _wasm() {
        return window.wasmBindings || null;
    }

    _analyzeAndPublish() {
        const wasm = this._wasm();
        if (!wasm || typeof wasm.analyze_lex !== 'function') {
            this.lex = { tokens: [], diagnostics: [] };
            this.parse = null;
            this.resolve = null;
            this.semantics = null;
            this.updateCallback({
                tokens: [],
                diagnostics: [],
                foldingRanges: [],
                config: { highlightWhitespace: true, highlightIndent: true },
            });
            return;
        }

        try {
            this.lex = wasm.analyze_lex(this.text);
            this.parse = typeof wasm.analyze_parse === 'function' ? wasm.analyze_parse(this.text) : null;
            this.resolve = typeof wasm.analyze_name_resolution === 'function' ? wasm.analyze_name_resolution(this.text) : null;
            this.semantics = typeof wasm.analyze_semantics === 'function' ? wasm.analyze_semantics(this.text) : null;
        } catch (e) {
            console.error('[NEPLg2LanguageProvider] analyze failed:', e);
            this.lex = { tokens: [], diagnostics: [] };
            this.parse = null;
            this.resolve = null;
            this.semantics = null;
        }

        const tokens = this._buildEditorTokens();
        const diagnostics = this._collectDiagnostics();
        const foldingRanges = this._buildFoldingRanges();

        this.updateCallback({
            tokens,
            diagnostics,
            foldingRanges,
            config: { highlightWhitespace: true, highlightIndent: true },
        });
    }

    _spanFrom(obj) {
        const s = obj && obj.span;
        if (!s) return null;
        return {
            startIndex: Number(s.start ?? 0),
            endIndex: Number(s.end ?? 0),
            startLine: Number(s.start_line ?? 0),
            endLine: Number(s.end_line ?? 0),
        };
    }

    _severity(diag) {
        const sv = String(diag?.severity || 'error').toLowerCase();
        return sv.includes('warn') ? 'warning' : 'error';
    }

    _collectDiagnostics() {
        const all = [];
        const pushFrom = (arr) => {
            if (!Array.isArray(arr)) return;
            for (const d of arr) {
                const sp = this._spanFrom(d);
                all.push({
                    startIndex: sp ? sp.startIndex : 0,
                    endIndex: sp ? sp.endIndex : 0,
                    message: String(d?.message || 'diagnostic'),
                    severity: this._severity(d),
                });
            }
        };

        pushFrom(this.lex?.diagnostics);
        pushFrom(this.parse?.diagnostics);
        pushFrom(this.parse?.lex_diagnostics);
        pushFrom(this.resolve?.diagnostics);
        pushFrom(this.semantics?.diagnostics);

        all.sort((a, b) => a.startIndex - b.startIndex || a.endIndex - b.endIndex);
        return all;
    }

    _tokenType(kind, debug) {
        if (!kind) return 'default';
        if (kind.startsWith('Kw') || kind === 'At' || kind === 'PathSep') return 'keyword';
        if (kind.includes('String') || kind.includes('Mlstr')) return 'string';
        if (kind.includes('BoolLiteral')) return 'boolean';
        if (kind.includes('IntLiteral') || kind.includes('FloatLiteral')) return 'number';
        if (kind.includes('Comment')) return 'comment';
        if (kind === 'Ident') return 'variable';
        if (kind === 'Pipe' || kind === 'Arrow' || kind === 'Plus' || kind === 'Minus' || kind === 'Star' || kind === 'Slash' || kind === 'Equals') return 'operator';
        if (kind === 'LParen' || kind === 'RParen' || kind === 'LAngle' || kind === 'RAngle' || kind === 'Colon' || kind === 'Semicolon' || kind === 'Comma' || kind === 'Dot') return 'punctuation';
        if (debug && String(debug).includes('Fn')) return 'function';
        return 'default';
    }

    _buildEditorTokens() {
        const lexTokens = Array.isArray(this.lex?.tokens) ? this.lex.tokens : [];
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        const defById = new Map(defs.map((d) => [d.id, d]));

        return lexTokens.map((tok, idx) => {
            const span = this._spanFrom(tok) || { startIndex: 0, endIndex: 0 };
            let t = this._tokenType(String(tok.kind || ''), tok.debug);

            const tr = tokenRes[idx];
            if (tr && tr.resolved_def_id != null) {
                const def = defById.get(tr.resolved_def_id);
                if (def && (def.kind === 'fn' || def.kind === 'fn_alias')) {
                    t = 'function';
                }
            }
            return {
                startIndex: span.startIndex,
                endIndex: span.endIndex,
                type: t,
            };
        });
    }

    _walkAst(node, out) {
        if (!node || typeof node !== 'object') return;
        if (node.kind === 'Block' && node.span && Number(node.span.end_line) > Number(node.span.start_line)) {
            out.push({
                startLine: Number(node.span.start_line),
                endLine: Number(node.span.end_line),
                placeholder: '...'
            });
        }
        for (const v of Object.values(node)) {
            if (Array.isArray(v)) {
                for (const it of v) this._walkAst(it, out);
            } else if (v && typeof v === 'object') {
                this._walkAst(v, out);
            }
        }
    }

    _buildFoldingRanges() {
        const root = this.parse?.module?.root;
        if (!root) return [];
        const ranges = [];
        this._walkAst(root, ranges);
        ranges.sort((a, b) => a.startLine - b.startLine || a.endLine - b.endLine);
        return ranges;
    }

    _tokenAt(index) {
        const tokens = Array.isArray(this.lex?.tokens) ? this.lex.tokens : [];
        for (let i = 0; i < tokens.length; i++) {
            const sp = this._spanFrom(tokens[i]);
            if (sp && index >= sp.startIndex && index < sp.endIndex) {
                return { token: tokens[i], tokenIndex: i, span: sp };
            }
        }
        return null;
    }

    _tokenSemanticByIndex(tokenIndex) {
        const tokenSem = Array.isArray(this.semantics?.token_semantics) ? this.semantics.token_semantics : [];
        return tokenSem.find((x) => Number(x?.token_index) === tokenIndex) || null;
    }

    _tokenResolutionByIndex(tokenIndex) {
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        return tokenRes.find((x) => Number(x?.token_index) === tokenIndex) || null;
    }

    _formatSpan(sp) {
        if (!sp) return null;
        const s = Number(sp.start ?? 0);
        const e = Number(sp.end ?? 0);
        return `[${s}, ${e})`;
    }

    getTokenInsight(index) {
        const hit = this._tokenAt(index);
        if (!hit) return null;

        const ts = this._tokenSemanticByIndex(hit.tokenIndex);
        const tr = this._tokenResolutionByIndex(hit.tokenIndex);
        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        const def = tr && tr.resolved_def_id != null ? defs.find((d) => d.id === tr.resolved_def_id) : null;

        return {
            tokenIndex: hit.tokenIndex,
            tokenKind: String(hit.token?.kind || ''),
            tokenSpan: hit.span,
            inferredType: ts?.inferred_type || null,
            exprSpan: ts?.expr_span || null,
            argIndex: Number.isInteger(ts?.arg_index) ? ts.arg_index : null,
            argSpan: ts?.arg_span || null,
            resolvedDefId: tr?.resolved_def_id ?? null,
            candidateDefIds: Array.isArray(tr?.candidate_def_ids) ? tr.candidate_def_ids : [],
            resolvedDefinition: def
                ? {
                    id: def.id,
                    name: def.name,
                    kind: def.kind,
                    span: def.span || null,
                }
                : null,
        };
    }

    async getHoverInfo(index) {
        const hit = this._tokenAt(index);
        if (!hit) return null;

        const tokenSem = Array.isArray(this.semantics?.token_semantics) ? this.semantics.token_semantics : [];
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        const defById = new Map(defs.map((d) => [d.id, d]));

        const ts = tokenSem.find((x) => Number(x?.token_index) === hit.tokenIndex) || null;
        const tr = tokenRes.find((x) => Number(x?.token_index) === hit.tokenIndex) || null;

        const lines = [];
        const raw = String(hit.token?.value || hit.token?.debug || '').trim();
        if (raw) lines.push(raw);
        if (ts && ts.inferred_type) lines.push(`type: ${ts.inferred_type}`);
        if (ts && ts.expr_span) lines.push(`expr: ${this._formatSpan(ts.expr_span)}`);
        if (ts && Number.isInteger(ts.arg_index)) lines.push(`arg#${ts.arg_index}: ${this._formatSpan(ts.arg_span)}`);
        if (tr && tr.resolved_def_id != null) {
            const def = defById.get(tr.resolved_def_id);
            if (def) lines.push(`def: ${def.kind} ${def.name}`);
        }
        if (tr && Array.isArray(tr.candidate_def_ids) && tr.candidate_def_ids.length > 1) {
            lines.push(`candidates: ${tr.candidate_def_ids.join(', ')}`);
        }
        if (lines.length === 0) return null;
        return { content: lines.join('\n'), startIndex: hit.span.startIndex, endIndex: hit.span.endIndex };
    }

    async getDefinitionLocation(index) {
        const hit = this._tokenAt(index);
        if (!hit) return null;
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        const tr = tokenRes.find((x) => Number(x?.token_index) === hit.tokenIndex);
        if (!tr || tr.resolved_def_id == null) return null;
        const def = defs.find((d) => d.id === tr.resolved_def_id);
        if (!def || !def.span) return null;
        return { targetIndex: Number(def.span.start ?? 0) };
    }

    async getOccurrences(index) {
        const hit = this._tokenAt(index);
        if (!hit) return [];
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        const refs = Array.isArray(this.resolve?.references) ? this.resolve.references : [];
        const tr = tokenRes.find((x) => Number(x?.token_index) === hit.tokenIndex);
        if (!tr) return [];

        const out = [];
        for (const r of refs) {
            if (tr.resolved_def_id != null && r.resolved_def_id === tr.resolved_def_id && r.span) {
                out.push({ startIndex: Number(r.span.start ?? 0), endIndex: Number(r.span.end ?? 0) });
            }
        }
        if (out.length === 0 && tr.name) {
            for (const r of refs) {
                if (r.name === tr.name && r.span) {
                    out.push({ startIndex: Number(r.span.start ?? 0), endIndex: Number(r.span.end ?? 0) });
                }
            }
        }
        return out;
    }

    _wordAt(index) {
        const s = this.text || '';
        let l = index;
        let r = index;
        const isWord = (c) => /[A-Za-z0-9_#]/.test(c);
        while (l > 0 && isWord(s[l - 1])) l--;
        while (r < s.length && isWord(s[r])) r++;
        return { start: l, end: r, text: s.slice(l, r) };
    }

    async getNextWordBoundary(index, direction) {
        const s = this.text || '';
        if (direction === 'left') {
            let i = Math.max(0, index - 1);
            while (i > 0 && /\s/.test(s[i])) i--;
            while (i > 0 && /[A-Za-z0-9_]/.test(s[i - 1])) i--;
            return { targetIndex: i };
        }
        let i = Math.min(s.length, index);
        while (i < s.length && /[A-Za-z0-9_]/.test(s[i])) i++;
        while (i < s.length && /\s/.test(s[i])) i++;
        return { targetIndex: i };
    }

    async getCompletions(index) {
        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        const word = this._wordAt(index);
        const prefix = (word?.text || '').toLowerCase();

        const items = [];
        for (const kw of this.keywordCompletions) {
            items.push({ label: kw, type: 'keyword', insertText: kw });
        }
        for (const d of defs) {
            items.push({
                label: String(d.name || ''),
                type: d.kind === 'fn' || d.kind === 'fn_alias' ? 'function' : 'variable',
                insertText: String(d.name || ''),
                detail: String(d.kind || ''),
            });
        }

        if (!prefix) return items;
        return items.filter((it) => String(it.label || '').toLowerCase().startsWith(prefix));
    }

    async getIndentation(index) {
        const lineStart = this.text.lastIndexOf('\n', index - 1) + 1;
        const line = this.text.slice(lineStart, index);
        const indent = (line.match(/^\s*/) || [''])[0];
        const trimmed = line.trim();
        if (trimmed.endsWith(':')) {
            return { textToInsert: `\n${indent}    `, cursorOffset: indent.length + 5 };
        }
        return { textToInsert: `\n${indent}`, cursorOffset: indent.length + 1 };
    }

    async toggleComment(selectionStart, selectionEnd) {
        const lineStart = this.text.lastIndexOf('\n', selectionStart - 1) + 1;
        let lineEnd = this.text.indexOf('\n', selectionEnd);
        if (lineEnd === -1) lineEnd = this.text.length;

        const selected = this.text.slice(lineStart, lineEnd);
        const lines = selected.split('\n');
        const allCommented = lines.filter((l) => l.trim() !== '').every((l) => l.trimStart().startsWith('//'));

        const next = lines.map((line) => {
            if (line.trim() === '') return line;
            if (allCommented) return line.replace(/^(\s*)\/\/\s?/, '$1');
            const lead = (line.match(/^\s*/) || [''])[0];
            return `${lead}// ${line.slice(lead.length)}`;
        });

        const newText = this.text.slice(0, lineStart) + next.join('\n') + this.text.slice(lineEnd);
        return { newText, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd };
    }

    async adjustIndentation(selectionStart, selectionEnd, isOutdent) {
        const lines = this.text.split('\n');
        const indentUnit = '    ';
        let cursor = 0;
        let startLine = 0;
        let endLine = lines.length - 1;
        for (let i = 0; i < lines.length; i++) {
            const end = cursor + lines[i].length;
            if (selectionStart >= cursor && selectionStart <= end) startLine = i;
            if (selectionEnd >= cursor && selectionEnd <= end) {
                endLine = i;
                break;
            }
            cursor = end + 1;
        }

        for (let i = startLine; i <= endLine; i++) {
            if (isOutdent) {
                if (lines[i].startsWith(indentUnit)) lines[i] = lines[i].slice(indentUnit.length);
                else lines[i] = lines[i].replace(/^\s{1,4}/, '');
            } else {
                lines[i] = indentUnit + lines[i];
            }
        }

        const newText = lines.join('\n');
        return { newText, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd };
    }

    async getBracketMatch(index) {
        const text = this.text || '';
        const pairs = { '(': ')', '[': ']', '{': '}', ')': '(', ']': '[', '}': '{' };
        const c = text[index];
        if (!pairs[c]) return [];
        const isOpen = c === '(' || c === '[' || c === '{';
        const target = pairs[c];
        let depth = 1;
        for (let i = index + (isOpen ? 1 : -1); i >= 0 && i < text.length; i += isOpen ? 1 : -1) {
            if (text[i] === c) depth++;
            if (text[i] === target) depth--;
            if (depth === 0) {
                return [
                    { startIndex: index, endIndex: index + 1 },
                    { startIndex: i, endIndex: i + 1 },
                ];
            }
        }
        return [];
    }
}

window.NEPLg2LanguageProvider = NEPLg2LanguageProvider;
