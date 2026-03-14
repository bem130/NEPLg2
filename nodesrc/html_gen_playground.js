// nodesrc/html_gen_playground.js
// 目的:
// - 既存 html_gen.js は維持したまま、チュートリアル向けの実行可能 HTML を生成する。
// - 記事本体以外の UI (サイドバー、プレイグラウンド等) は playground_runtime.js が動的に生成する。
// - CSS も外部ファイル化されている。

const { renderNode, renderInlines } = require("./html_gen");
const { parseInlines } = require("./parser");
const { extractInPageToc, renderInPageTocHtml } = require("./inpage_toc_helper");
const fs = require("fs");
const path = require("path");

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderBody(ast) {
  return renderNode(ast, { rewriteLinks: true });
}

function renderTocItems(tocLinks) {
  if (!Array.isArray(tocLinks) || tocLinks.length === 0) {
    return "";
  }
  return tocLinks
    .map((link) => {
      const depth = Number.isFinite(link.depth)
        ? Math.max(0, Math.min(6, link.depth))
        : 0;

      const labelNodes = parseInlines(String(link.label || ""));
      const labelHtml = renderInlines(labelNodes);

      if (link.isGroup) {
        return `<li><div class="toc-group depth-${depth}">${labelHtml}</div></li>`;
      }
      const cls = link.active
        ? `toc-link active depth-${depth}`
        : `toc-link depth-${depth}`;
      return `<li><a class="${cls}" href="${escapeHtml(String(link.href || ""))}">${labelHtml}</a></li>`;
    })
    .join("\n");
}

function buildPlaygroundVfsOverrides() {
  const rels = [
    "stdlib/kp/kpread.nepl",
    "stdlib/kp/kpwrite.nepl",
    "stdlib/kp/kpgraph.nepl",
    "stdlib/kp/kpsearch.nepl",
    "stdlib/kp/kpprefix.nepl",
    "stdlib/kp/kpdsu.nepl",
    "stdlib/kp/kpfenwick.nepl",
  ];
  const out = {};
  for (const rel of rels) {
    const abs = path.resolve(process.cwd(), rel);
    if (!fs.existsSync(abs)) continue;
    const key = "/stdlib/" + rel.replace(/^stdlib\//, "").replace(/\\/g, "/");
    out[key] = fs.readFileSync(abs, "utf8");
  }
  return out;
}

function wrapHtmlPlayground(
  body,
  title,
  description,
  moduleJsPathOpt,
  runtimeJsPath,
  searchJsPath,
  playgroundCssPath,
  tocItemsHtml,
  tocTitle,
  searchIndexJson,
  rootPrefix,
  inPageTocHtml,
) {
  const t = title || "NEPLg2 Tutorial";
  const d =
    description || "NEPLg2 tutorial with interactive runnable examples.";
  const moduleJsPath =
    (moduleJsPathOpt && String(moduleJsPathOpt)) || "./nepl-web.js";
  const vfsOverrides = buildPlaygroundVfsOverrides();
  const vfsOverridesJson = JSON.stringify(vfsOverrides);
  const safeSearchIndexJson = searchIndexJson || "[]";
  const safeRootPrefix = rootPrefix || "./";
  const safeTocTitle = tocTitle || "Getting Started";

  return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${escapeHtml(t)}</title>
<meta name="description" content="${escapeHtml(d)}"/>
<meta property="og:title" content="${escapeHtml(t)}"/>
<meta property="og:description" content="${escapeHtml(d)}"/>
<meta property="og:site_name" content="NEPLg2"/>
<meta property="og:locale" content="ja_JP"/>
<meta property="og:image" content="https://neknaj.github.io/NEPLg2/NEPLg2.png"/>
<meta property="og:type" content="article"/>
<meta name="twitter:card" content="summary"/>
<meta name="twitter:site" content="@bem130"/>
<meta name="twitter:creator" content="@bem130"/>
<meta name="twitter:title" content="${escapeHtml(t)}"/>
<meta name="twitter:description" content="${escapeHtml(d)}"/>
<meta name="twitter:image" content="https://neknaj.github.io/NEPLg2/NEPLg2.png"/>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Klee+One:wght@400;600&display=swap" rel="stylesheet">
<link rel="stylesheet" href="${playgroundCssPath}">
<script src="${searchJsPath}"></script>
<script type="module">
  import { initPlayground } from '${runtimeJsPath}';
  initPlayground({
    searchIndex: ${safeSearchIndexJson},
    rootPrefix: '${escapeHtml(safeRootPrefix)}',
    vfsOverrides: ${vfsOverridesJson},
    moduleJsPath: '${escapeHtml(moduleJsPath)}',
    tocHtml: \`${tocItemsHtml.replace(/`/g, '\\`').replace(/\$\{/g, '\\${')}\`,
    tocTitle: '${escapeHtml(safeTocTitle)}',
    title: '${escapeHtml(t)}'
  });
</script>
</head>
<body>
<div class="doc-layout">
<!-- Left Sidebar (dynamically populated by initPlayground) -->

<!-- Main Content -->
<main>
${body}
</main>

<!-- Right Sidebar (In-Page TOC) -->
${inPageTocHtml ? `<aside class="doc-inpage-toc"><div class="inpage-toc-inner"><div class="inpage-toc-title">ON THIS PAGE</div>${inPageTocHtml}</div></aside>` : ''}
</div>

<!-- Mobile In-Page TOC (Injected by JS if needed, or CSS-only visible on mobile layout) -->
${inPageTocHtml ? `<div class="doc-inpage-toc-mobile-container"><details class="doc-inpage-toc-mobile"><summary>On this page</summary>${inPageTocHtml}</details></div>` : ''}

</body>
</html>`;
}

function renderHtmlPlayground(ast, opt) {
  const title = opt && opt.title ? opt.title : "NEPLg2 Tutorial";
  const description =
    opt && opt.description
      ? opt.description
      : "NEPLg2 tutorial with interactive runnable examples.";
  const moduleJsPath =
    opt && opt.moduleJsPath ? String(opt.moduleJsPath) : "./nepl-web.js";
  
  const tocItemsHtml = renderTocItems(opt && opt.tocLinks ? opt.tocLinks : []);
  const tocTitle = opt && opt.tocTitle ? opt.tocTitle : "Getting Started";

  const searchIndex =
    opt && Array.isArray(opt.searchIndex) ? opt.searchIndex : [];
  const searchIndexJson = JSON.stringify(searchIndex);
  const rootPrefix = opt && opt.rootPrefix ? String(opt.rootPrefix) : "./";
  const body = renderBody(ast);
  
  let inPageTocHtml = "";
  if (ast) {
      const rawTocNodes = extractInPageToc(ast);
      inPageTocHtml = renderInPageTocHtml(rawTocNodes);
  }

  return wrapHtmlPlayground(
    body,
    title,
    description,
    moduleJsPath,
    opt && opt.runtimeJsPath ? opt.runtimeJsPath : './playground_runtime.js',
    opt && opt.searchJsPath ? opt.searchJsPath : './search.js',
    opt && opt.playgroundCssPath ? opt.playgroundCssPath : './playground.css',
    tocItemsHtml,
    tocTitle,
    searchIndexJson,
    rootPrefix,
    inPageTocHtml
  );
}

module.exports = {
  renderHtmlPlayground,
};
