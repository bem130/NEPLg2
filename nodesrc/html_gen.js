/**
 * html_gen.js
 *
 * parser.js が生成した AST を静的 HTML に変換する。
 *
 * 目的:
 * - まずは「実行機能なし」の HTML を安定して生成することを優先する。
 * - tutorial などで、文書中の Gloss/Ruby/Nest が意図どおり表示されることを確認する。
 *
 * 仕様:
 * - 段落内の改行は <br/> に変換する（.n.md の改行規則）。
 * - CodeBlock は <pre><code> で出力し、既定で //:| 行を非表示にする（import 等を隠す用途）。
 *   （将来的には “クリックで展開” などに拡張可能）
 */

const escapeHtml = (s) =>
    s.replace(/&/g, "&amp;")
     .replace(/</g, "&lt;")
     .replace(/>/g, "&gt;")
     .replace(/"/g, "&quot;")
     .replace(/'/g, "&#39;");

function renderInlines(nodes) {
    let out = "";
    for (const n of nodes) {
        if (n.type === "text") {
            out += escapeHtml(n.text).replace(/\n/g, "<br/>");
        } else if (n.type === "math") {
            // MathJax 等は後段で足せるように、ここでは <span> で囲むだけ
            const cls = n.display ? "math-display" : "math-inline";
            out += `<span class="${cls}">${escapeHtml(n.text)}</span>`;
        } else if (n.type === "ruby") {
            out += `<ruby class="nm-ruby"><rb>${renderInlines(n.base)}</rb><rt>${renderInlines(n.ruby)}</rt></ruby>`;
        } else if (n.type === "gloss") {
            // 多言語は rt を複数行にする
            const base = renderInlines(n.base);
            const notes = n.notes.map(x => `<span class="nm-gloss-note">${renderInlines(x)}</span>`).join("");
            out += `<ruby class="nm-gloss"><rb>${base}</rb><rt>${notes}</rt></ruby>`;
        } else {
            out += escapeHtml(String(n));
        }
    }
    return out;
}

function renderNode(node, opt) {
    const o = opt || { hideDocPipe: true };

    if (node.type === "document") {
        return node.children.map(ch => renderNode(ch, o)).join("\n");
    }
    if (node.type === "section") {
        const tag = `h${Math.min(6, Math.max(1, node.level))}`;
        const head = renderInlines(node.heading);
        const body = node.children.map(ch => renderNode(ch, o)).join("\n");
        return `<section class="nm-sec level-${node.level}"><${tag}>${head}</${tag}>\n${body}\n</section>`;
    }
    if (node.type === "paragraph") {
        return `<p>${renderInlines(node.inlines)}</p>`;
    }
    if (node.type === "hr") {
        return `<hr/>`;
    }
    if (node.type === "list") {
        const items = node.items.map(it => `<li>${renderInlines(it)}</li>`).join("\n");
        return `<ul>\n${items}\n</ul>`;
    }
    if (node.type === "code") {
        let txt = node.text || "";
        if (o.hideDocPipe) {
            // //:| を含む行はデフォルト非表示
            txt = txt.split("\n").filter(ln => !/^\s*\/\/:\|\s?/.test(ln)).join("\n");
        }
        const cls = node.lang ? `language-${escapeHtml(node.lang)}` : "";
        return `<pre class="nm-code"><code class="${cls}">${escapeHtml(txt)}</code></pre>`;
    }
    return `<pre>${escapeHtml(JSON.stringify(node, null, 2))}</pre>`;
}

function wrapHtml(body, title) {
    const t = title || "nm";
    return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${escapeHtml(t)}</title>
<style>
:root{
  --bg:#0b0f19;
  --fg:#e6edf3;
  --muted:#aab6c3;
  --card:#121a2a;
  --border:#23304a;
  --code:#0f1626;
  --accent:#7aa2f7;
}
html,body{background:var(--bg);color:var(--fg);font-family:system-ui,-apple-system,Segoe UI,Roboto,Helvetica,Arial;line-height:1.65;}
main{max-width:980px;margin:24px auto;padding:0 16px;}
a{color:var(--accent);}
hr{border:none;border-top:1px solid var(--border);margin:24px 0;}
.nm-sec {padding: 0.5em;padding-left: 2em;margin: 1em;border-left: 3px solid var(--border);border-radius: 1em;}
h1,h2,h3,h4,h5,h6{margin:18px 0 10px;}
p{margin:10px 0;}
ul{margin:10px 0 10px 22px;}
.nm-code{background:var(--code);border:1px solid var(--border);border-radius:12px;padding:12px;overflow:auto;}
.nm-code code{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;white-space:pre;}
.nm-gloss, .nm-ruby{ruby-position:over;}
.nm-gloss rt{font-size:0.72em;color:var(--muted);line-height:1.1;}
.nm-gloss-note{display:block;}
.math-inline{color:var(--muted);}
.math-display{display:block;padding:8px 10px;margin:8px 0;background:rgba(255,255,255,0.03);border:1px dashed var(--border);border-radius:10px;}
</style>
</head>
<body>
<main>
${body}
</main>
</body>
</html>`;
}

function renderHtml(ast, opt) {
    const body = renderNode(ast, opt);
    return wrapHtml(body, opt && opt.title ? opt.title : "nm");
}

module.exports = {
    renderHtml,
    renderNode,
    renderInlines,
};
