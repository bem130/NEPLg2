/**
 * parser.js
 *
 * .n.md (Gloss + Nest 拡張Markdown) を AST に変換するパーサ。
 *
 * 目的:
 * - README.n.md に書かれている拡張記法（Gloss, ruby, Nest, break ルール, コードフェンス等）を
 *   Node.js 側で解釈できるようにする。
 *
 * 設計方針:
 * - Markdown 全機能を目指さず、「nm が必要とする最小集合」をまず正確に実装する。
 * - AST を先に作り、HTML 生成は別モジュール (html_gen.js) に分離する。
 *
 * 仕様の要点:
 * - 見出し (#..######) により Section をネストする（Nest）。
 * - '---' は <hr/> を出し、さらに Section を 1 段だけ close する。
 * - ';;;' は表示せず Section を 1 段だけ close する。
 * - '----' 以上のハイフンは <hr/> として扱うが Nest には干渉しない。
 * - 改行:
 *   - .n.md 内の実改行は改行として表示される。
 *   - 文字列としての '\n' も改行として表示される（README の規則）。
 */

function isHeadingLine(line) {
    const m = /^(#{1,6})\s+(.*)$/.exec(line);
    if (!m) return null;
    return { level: m[1].length, text: m[2] };
}

function isFenceStart(line) {
    const m = /^```([^\s`]*)\s*$/.exec(line);
    if (!m) return null;
    return { lang: m[1] || "" };
}

function isFenceEnd(line) {
    return /^```\s*$/.test(line);
}

function isBreakClose1(line) {
    return line === ';;;';
}

function isBreakHrClose1(line) {
    return line === '---';
}

function isHrOnly(line) {
    // ---- 以上は hr だが Nest に干渉しない（README の規則）
    return /^-{4,}\s*$/.test(line);
}

function normalizeNewlinesLiteral(s) {
    // 文字列としての "\n" を実改行にする
    return s.replace(/\\n/g, "\n");
}

// インライン解析（最小実装）
// - Math: $...$ / $$...$$ は中身を保護
// - Ruby: [漢字/かんじ] ただし [text](url) はリンクとみなし ruby 解析しない
// - Gloss: {A/b/β} ただし [] 内の / は区切りにしない
function parseInlines(text) {
    const s = normalizeNewlinesLiteral(text);
    const out = [];
    let i = 0;

    const pushText = (t) => {
        if (!t) return;
        const prev = out[out.length - 1];
        if (prev && prev.type === "text") prev.text += t;
        else out.push({ type: "text", text: t });
    };

    const readUntil = (needle, start) => {
        const idx = s.indexOf(needle, start);
        if (idx === -1) return null;
        return { end: idx, content: s.slice(start, idx) };
    };

    const splitGlossParts = (inner) => {
        // [] の中では / を区切りとして扱わない
        const parts = [];
        let buf = "";
        let bracket = 0;
        for (let k = 0; k < inner.length; k++) {
            const ch = inner[k];
            if (ch === '[') bracket++;
            if (ch === ']') bracket = Math.max(0, bracket - 1);
            if (ch === '/' && bracket === 0) {
                parts.push(buf);
                buf = "";
                continue;
            }
            buf += ch;
        }
        parts.push(buf);
        return parts;
    };

    while (i < s.length) {
        // $$...$$
        if (s.startsWith("$$", i)) {
            const r = readUntil("$$", i + 2);
            if (r) {
                out.push({ type: "math", display: true, text: r.content });
                i = r.end + 2;
                continue;
            }
        }
        // $...$
        if (s[i] === '$') {
            const r = readUntil("$", i + 1);
            if (r) {
                out.push({ type: "math", display: false, text: r.content });
                i = r.end + 1;
                continue;
            }
        }
        // Gloss { ... }
        if (s[i] === '{') {
            const r = readUntil("}", i + 1);
            if (r) {
                const parts = splitGlossParts(r.content).map(p => p.trim());
                if (parts.length >= 2) {
                    out.push({
                        type: "gloss",
                        base: parseInlines(parts[0]),
                        notes: parts.slice(1).map(p => parseInlines(p)),
                    });
                    i = r.end + 1;
                    continue;
                }
                // parts が 1 個なら通常テキストとして扱う
            }
        }
        // Ruby [base/ruby] ただし [text](url) は除外
        if (s[i] === '[') {
            const r = readUntil("]", i + 1);
            if (r) {
                const after = s[r.end + 1] || "";
                if (after === '(') {
                    // Markdown link の可能性: ruby にしない
                    pushText(s.slice(i, r.end + 1));
                    i = r.end + 1;
                    continue;
                }
                const inner = r.content;
                const slash = inner.indexOf("/");
                if (slash !== -1) {
                    const base = inner.slice(0, slash);
                    const ruby = inner.slice(slash + 1);
                    out.push({
                        type: "ruby",
                        base: parseInlines(base),
                        ruby: parseInlines(ruby),
                    });
                    i = r.end + 1;
                    continue;
                }
            }
        }

        // 既定: 1 文字進める
        pushText(s[i]);
        i += 1;
    }

    return out;
}

function newDocument() {
    return { type: "document", children: [] };
}

function newSection(level, headingInlines) {
    return { type: "section", level, heading: headingInlines, children: [] };
}

function newParagraph(lines) {
    // 段落内の改行は保持する（HTML 側で <br/> 化）
    return { type: "paragraph", inlines: parseInlines(lines.join("\n")) };
}

function newHr() {
    return { type: "hr" };
}

function newCodeBlock(lang, codeText) {
    return { type: "code", lang, text: codeText };
}

function parseNmd(source) {
    const lines = source.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");

    const doc = newDocument();
    const stack = [{ level: 0, node: doc }]; // section stack: doc is level 0
    const curContainer = () => stack[stack.length - 1].node;

    const closeToLevel = (level) => {
        while (stack.length > 1 && stack[stack.length - 1].level >= level) {
            stack.pop();
        }
    };

    const closeOne = () => {
        if (stack.length > 1) stack.pop();
    };

    let i = 0;
    while (i < lines.length) {
        const line = lines[i];

        // Fence
        const fs = isFenceStart(line);
        if (fs) {
            let j = i + 1;
            const codeLines = [];
            while (j < lines.length && !isFenceEnd(lines[j])) {
                codeLines.push(lines[j]);
                j += 1;
            }
            // closing fence を 1 行消費
            if (j < lines.length && isFenceEnd(lines[j])) j += 1;

            curContainer().children.push(newCodeBlock(fs.lang, codeLines.join("\n")));
            i = j;
            continue;
        }

        // Heading
        const h = isHeadingLine(line);
        if (h) {
            closeToLevel(h.level);
            const sec = newSection(h.level, parseInlines(h.text));
            curContainer().children.push(sec);
            stack.push({ level: h.level, node: sec });
            i += 1;
            continue;
        }

        // Breaks / Hr
        if (isBreakHrClose1(line)) {
            curContainer().children.push(newHr());
            closeOne();
            i += 1;
            continue;
        }
        if (isBreakClose1(line)) {
            closeOne();
            i += 1;
            continue;
        }
        if (isHrOnly(line)) {
            curContainer().children.push(newHr());
            i += 1;
            continue;
        }

        // Blank
        if (line.trim() === "") {
            i += 1;
            continue;
        }

        // List item (minimal)
        const lm = /^-\s+(.*)$/.exec(line);
        if (lm) {
            // 連続する - item をまとめて list にする
            const items = [];
            let j = i;
            while (j < lines.length) {
                const m2 = /^-\s+(.*)$/.exec(lines[j]);
                if (!m2) break;
                items.push(parseInlines(m2[1]));
                j += 1;
            }
            curContainer().children.push({ type: "list", items });
            i = j;
            continue;
        }

        // Paragraph
        const para = [];
        let j = i;
        while (j < lines.length) {
            const ln = lines[j];
            if (ln.trim() === "") break;
            if (isFenceStart(ln) || isHeadingLine(ln) || isBreakHrClose1(ln) || isBreakClose1(ln) || isHrOnly(ln) || /^-\s+/.test(ln)) break;
            para.push(ln);
            j += 1;
        }
        curContainer().children.push(newParagraph(para));
        i = j;
    }

    return doc;
}

module.exports = {
    parseNmd,
    parseInlines,
};
