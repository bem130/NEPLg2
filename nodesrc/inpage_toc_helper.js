const { parseInlines } = require("./parser");

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeHtmlAttr(s) { return escapeHtml(s); }

function inlinesToPlainText(inlines) {
  if (!Array.isArray(inlines)) return "";
  return inlines
    .map((n) => {
      if (n.type === "text") return n.text;
      if (n.type === "code_inline") return n.text;
      if (n.type === "math") return n.text;
      if (n.type === "ruby") {
        return inlinesToPlainText(n.base) + " " + inlinesToPlainText(n.ruby);
      }
      if (n.type === "gloss") {
        const base = inlinesToPlainText(n.base);
        const notes = (n.notes || [])
          .map((x) => inlinesToPlainText(x))
          .join(" ");
        return base + " " + notes;
      }
      if (n.type === "link") return inlinesToPlainText(n.text);
      return "";
    })
    .join("")
    .replace(/\s+/g, " ")
    .trim();
}

function makeSlug(text, typeInfo) {
  let base = text
    .toLowerCase()
    .replace(/[\s\u3000]+/g, "-")
    .replace(/[^\w\u3040-\u309f\u30a0-\u30ff\u4e00-\u9fff\uac00-\ud7af\-]/g, "");
  if (typeInfo) {
    const safeType = typeInfo
      .replace(/\s+/g, "")
      .replace(/[<>()*]/g, "")
      .replace(/[,]/g, "-")
      .replace(/->/g, "-to-")
      .toLowerCase();
    base += "--" + safeType;
  }
  return base.slice(0, 100);
}

function extractInPageToc(node, ancestorSlug = "") {
  let tocNodes = [];
  
  if (node.type === "document") {
    for (const child of node.children) {
      tocNodes.push(...extractInPageToc(child, ancestorSlug));
    }
  } else if (node.type === "section") {
    const titleText = inlinesToPlainText(node.heading);
    const slug = makeSlug(titleText, node.typeInfo);
    const fullId = ancestorSlug ? ancestorSlug + "-" + slug : slug;
    
    // NOTE: Ignore level 1 as it's the main page title. Level 2 and below belong in the TOC.
    if (node.level > 1 || true) { // We can filter later or just include
      tocNodes.push({
        id: fullId,
        level: node.level,
        title: titleText,
        typeInfo: node.typeInfo,
        kind: node.kind,
      });
    }
    
    for (const child of node.children) {
      tocNodes.push(...extractInPageToc(child, fullId));
    }
  }
  
  return tocNodes;
}

function renderInPageTocHtml(tocNodes) {
  if (!tocNodes || tocNodes.length === 0) return "";
  
  // Exclude level 1 headings from in-page TOC (typically the document title)
  const filtered = tocNodes.filter(n => n.level > 1);
  if (filtered.length === 0) return "";

  const minLevel = Math.min(...filtered.map(n => n.level));
  
  const items = filtered.map(node => {
     // Adjust depth relative to the minimum heading level found
     let depth = node.level - minLevel;
     if (depth < 0) depth = 0;
     if (depth > 6) depth = 6;
     
     let badgeHtml = "";
     if (node.kind) {
       badgeHtml = ` <span class="nm-badge nm-badge-${escapeHtmlAttr(node.kind)} inpage-toc-badge">${escapeHtml(node.kind)}</span>`;
     }
     
     return `<li><a class="inpage-toc-link depth-${depth}" href="#${escapeHtmlAttr(node.id)}">${escapeHtml(node.title)}${badgeHtml}</a></li>`;
  }).join("\n");
  
  return `<ul class="inpage-toc-list">\n${items}\n</ul>`;
}

module.exports = {
  extractInPageToc,
  renderInPageTocHtml
};
