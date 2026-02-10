// nodesrc/html_gen_playground.js
// 目的:
// - 既存 html_gen.js は維持したまま、チュートリアル向けの実行可能 HTML を生成する。
// - pre>code(language-neplg2) をクリックすると、ポップアップエディタで Run / Interrupt / 出力確認ができる。

const { renderNode } = require('./html_gen');

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

function renderBody(ast) {
    return renderNode(ast, { rewriteLinks: true });
}

function wrapHtmlPlayground(body, title, description, moduleJsPathOpt) {
    const t = title || 'NEPLg2 Tutorial';
    const d = description || 'NEPLg2 tutorial with interactive runnable examples.';
    const moduleJsPath = (moduleJsPathOpt && String(moduleJsPathOpt)) || './nepl-web.js';
    return `<!doctype html>
<html lang="ja">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>${escapeHtml(t)}</title>
<meta name="description" content="${escapeHtml(d)}"/>
<meta property="og:title" content="${escapeHtml(t)}"/>
<meta property="og:description" content="${escapeHtml(d)}"/>
<meta property="og:type" content="article"/>
<meta name="twitter:card" content="summary"/>
<meta name="twitter:title" content="${escapeHtml(t)}"/>
<meta name="twitter:description" content="${escapeHtml(d)}"/>
<style>
:root{
  --bg:#0b0f19;
  --fg:#e6edf3;
  --muted:#aab6c3;
  --card:#121a2a;
  --border:#23304a;
  --code:#0f1626;
  --accent:#7aa2f7;
  --ok:#59c37a;
  --err:#ff6b6b;
}
html,body{background:var(--bg);color:var(--fg);font-family:system-ui,-apple-system,Segoe UI,Roboto,Helvetica,Arial;line-height:1.65;}
main{max-width:980px;margin:24px auto;padding:0 16px;}
a{color:var(--accent);}
hr{border:none;border-top:1px solid var(--border);margin:24px 0;}
.nm-sec{padding:0.5em;padding-left:2em;margin:1em;border-left:3px solid var(--border);border-radius:1em;}
h1,h2,h3,h4,h5,h6{margin:18px 0 10px;}
p{margin:10px 0;}
ul{margin:10px 0 10px 22px;}
.nm-code{background:var(--code);border:1px solid var(--border);border-radius:12px;padding:12px;overflow:auto;}
.nm-code code{font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;white-space:pre;}
.nm-code-inline{background:rgba(255,255,255,0.06);border:1px solid rgba(255,255,255,0.10);border-radius:8px;padding:1px 6px;}
.nm-gloss, .nm-ruby{ruby-position:over;}
.nm-gloss rt{font-size:0.72em;color:var(--muted);line-height:1.1;}
.nm-gloss-note{display:block;}
.math-inline{color:var(--muted);}
.math-display{display:block;padding:8px 10px;margin:8px 0;background:rgba(255,255,255,0.03);border:1px dashed var(--border);border-radius:10px;}
.nm-doctest-meta{display:inline-block;margin:8px 0 2px;padding:3px 10px;border:1px solid var(--border);border-radius:999px;color:var(--muted);font-size:12px;background:rgba(255,255,255,0.03);}
.nm-toggle{display:inline-block;margin:6px 0 12px;padding:6px 10px;border-radius:10px;border:1px solid #2f3f58;background:#0f141b;color:#d6d6d6;cursor:pointer;}
.nm-hidden{display:none;}
.nm-runnable{cursor:pointer;position:relative;}
.nm-runnable::after{
  content:"Click to run";
  position:absolute;
  right:10px;
  top:8px;
  font-size:11px;
  color:var(--muted);
  background:rgba(0,0,0,.35);
  border:1px solid var(--border);
  border-radius:8px;
  padding:2px 6px;
}
#play-overlay{
  position:fixed; inset:0; background:rgba(0,0,0,.55);
  display:none; align-items:center; justify-content:center; z-index:9999;
}
#play-overlay.open{display:flex;}
#play-modal{
  width:min(1100px,95vw); height:min(760px,92vh);
  background:var(--card); border:1px solid var(--border); border-radius:12px;
  display:grid; grid-template-rows:auto 1fr auto; overflow:hidden;
}
#play-head,#play-foot{display:flex; align-items:center; gap:8px; padding:10px 12px; border-bottom:1px solid var(--border);}
#play-foot{border-bottom:none; border-top:1px solid var(--border);}
#play-title{font-weight:600; flex:1;}
.play-btn{padding:6px 10px; border-radius:8px; border:1px solid var(--border); background:#0f141b; color:var(--fg); cursor:pointer;}
.play-btn:hover{border-color:#355186;}
#play-editor{
  display:grid; grid-template-columns:1fr 40%;
  min-height:0;
}
#play-src,#play-stdin,#play-stdout{
  width:100%; height:100%; resize:none; box-sizing:border-box;
  font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;
  font-size:13px; line-height:1.45; border:none; outline:none; color:var(--fg); background:#0b1322;
  padding:12px;
}
#play-right{display:grid; grid-template-rows:120px 1fr; border-left:1px solid var(--border); min-height:0;}
#play-stdin{background:#0a1620; border-bottom:1px solid var(--border);}
#play-stdout{background:#081018;}
#play-status{font-size:12px; color:var(--muted);}
.ok{color:var(--ok);} .err{color:var(--err);}
</style>
<script>
function nmToggleHidden(btn){
  const pre = btn.previousElementSibling;
  if(!pre) return;
  const nodes = pre.querySelectorAll('.nm-hidden');
  if(nodes.length === 0) return;
  const cur = nodes[0].style.display;
  const show = (cur === 'none' || cur === '');
  for(const n of nodes){
    n.style.display = show ? 'inline' : 'none';
  }
  btn.textContent = show ? '前置き( | 行)を隠す' : '前置き( | 行)を表示';
}

async function loadBindings() {
  if (window.wasmBindings && typeof window.wasmBindings.compile_source === 'function') {
    return window.wasmBindings;
  }
  const modUrl = new URL('${escapeHtml(moduleJsPath)}', location.href).toString();
  const mod = await import(modUrl);
  if (typeof mod.default === 'function') {
    await mod.default();
  }
  window.wasmBindings = mod;
  return mod;
}

function makeWorkerScript() {
  return \`
self.onmessage = async (e) => {
  const { wasmBytes, stdinText } = e.data;
  let memory = null;
  let stdinOffset = 0;
  const stdin = new TextEncoder().encode(stdinText || '');
  const wasi = {
    fd_write(fd, iovs, iovs_len, nwritten){
      if(!memory) return 5;
      const view = new DataView(memory.buffer);
      let total = 0;
      for(let i=0;i<iovs_len;i++){
        const ptr = view.getUint32(iovs + i*8, true);
        const len = view.getUint32(iovs + i*8 + 4, true);
        const bytes = new Uint8Array(memory.buffer, ptr, len);
        self.postMessage({type:'stdout', fd, text:new TextDecoder().decode(bytes)});
        total += len;
      }
      view.setUint32(nwritten, total, true);
      return 0;
    },
    fd_read(fd, iovs, iovs_len, nread){
      if(fd !== 0) return 0;
      if(!memory) return 5;
      const view = new DataView(memory.buffer);
      let read = 0;
      for(let i=0;i<iovs_len;i++){
        const ptr = view.getUint32(iovs + i*8, true);
        const len = view.getUint32(iovs + i*8 + 4, true);
        const remain = stdin.length - stdinOffset;
        const take = Math.min(len, Math.max(0, remain));
        if (take > 0) {
          new Uint8Array(memory.buffer, ptr, take).set(stdin.subarray(stdinOffset, stdinOffset + take));
          stdinOffset += take;
          read += take;
        }
      }
      view.setUint32(nread, read, true);
      return 0;
    },
    fd_close(){ return 0; }, fd_seek(){ return 0; }, fd_fdstat_get(){ return 0; },
    environ_get(){ return 0; }, environ_sizes_get(){ return 0; },
    args_get(){ return 0; }, args_sizes_get(){ return 0; },
    clock_time_get(){ return 0; }, random_get(){ return 0; },
    proc_exit(code){ throw new Error('proc_exit:' + code); }
  };
  try {
    const { instance } = await WebAssembly.instantiate(wasmBytes, { wasi_snapshot_preview1: wasi });
    memory = instance.exports.memory;
    if (instance.exports._start) instance.exports._start();
    else if (instance.exports.main) instance.exports.main();
    self.postMessage({ type: 'done' });
  } catch (err) {
    self.postMessage({ type: 'error', message: String(err && err.message || err) });
  }
};
\`;
}

window.addEventListener('DOMContentLoaded', () => {
  for(const pre of document.querySelectorAll('pre.nm-code')){
    const hasHidden = pre.querySelector('.nm-hidden');
    if(!hasHidden) continue;
    for(const n of pre.querySelectorAll('.nm-hidden')){
      n.style.display = 'none';
    }
    const btn = document.createElement('button');
    btn.className = 'nm-toggle';
    btn.textContent = '前置き( | 行)を表示';
    btn.onclick = () => nmToggleHidden(btn);
    pre.insertAdjacentElement('afterend', btn);
  }

  const overlay = document.getElementById('play-overlay');
  const title = document.getElementById('play-title');
  const src = document.getElementById('play-src');
  const stdin = document.getElementById('play-stdin');
  const stdout = document.getElementById('play-stdout');
  const status = document.getElementById('play-status');
  const runBtn = document.getElementById('play-run');
  const stopBtn = document.getElementById('play-stop');
  const closeBtn = document.getElementById('play-close');
  let worker = null;
  let running = false;

  function setStatus(text, cls) {
    status.className = cls || '';
    status.textContent = text;
  }

  function stopRun(message) {
    if (worker) {
      worker.terminate();
      worker = null;
    }
    running = false;
    if (message) setStatus(message, 'err');
  }

  runBtn.onclick = async () => {
    if (running) return;
    stdout.value = '';
    setStatus('compiling...', '');
    try {
      const bindings = await loadBindings();
      const wasmBytes = bindings.compile_source(src.value);
      setStatus('running...', '');
      const blob = new Blob([makeWorkerScript()], { type: 'text/javascript' });
      worker = new Worker(URL.createObjectURL(blob));
      running = true;
      worker.onmessage = (ev) => {
        const msg = ev.data || {};
        if (msg.type === 'stdout') {
          stdout.value += String(msg.text || '');
          stdout.scrollTop = stdout.scrollHeight;
        } else if (msg.type === 'done') {
          running = false;
          setStatus('done', 'ok');
          worker && worker.terminate();
          worker = null;
        } else if (msg.type === 'error') {
          running = false;
          setStatus('runtime error', 'err');
          stdout.value += '\\n[error] ' + String(msg.message || '');
          worker && worker.terminate();
          worker = null;
        }
      };
      worker.postMessage({ wasmBytes, stdinText: stdin.value || '' });
    } catch (e) {
      running = false;
      setStatus('compile failed', 'err');
      stdout.value += '[compile error] ' + String((e && e.message) || e);
    }
  };

  stopBtn.onclick = () => {
    if (!running) return;
    stopRun('interrupted');
  };

  closeBtn.onclick = () => {
    stopRun('');
    overlay.classList.remove('open');
  };

  overlay.addEventListener('click', (ev) => {
    if (ev.target === overlay) {
      closeBtn.onclick();
    }
  });

  for (const code of document.querySelectorAll('pre.nm-code > code.language-neplg2')) {
    const pre = code.parentElement;
    pre.classList.add('nm-runnable');
    pre.title = 'Click to run in popup editor';
    pre.addEventListener('click', () => {
      title.textContent = document.title + ' - runnable snippet';
      src.value = code.textContent || '';
      stdin.value = '';
      stdout.value = '';
      setStatus('ready', 'ok');
      overlay.classList.add('open');
      src.focus();
    });
  }
});
</script>
</head>
<body>
<main>
${body}
</main>

<div id="play-overlay">
  <div id="play-modal" role="dialog" aria-modal="true" aria-label="NEPLg2 Runnable Snippet">
    <div id="play-head">
      <div id="play-title">Runnable Snippet</div>
      <button id="play-run" class="play-btn">Run</button>
      <button id="play-stop" class="play-btn">Interrupt</button>
      <button id="play-close" class="play-btn">Close</button>
    </div>
    <div id="play-editor">
      <textarea id="play-src" spellcheck="false"></textarea>
      <div id="play-right">
        <textarea id="play-stdin" spellcheck="false" placeholder="stdin"></textarea>
        <textarea id="play-stdout" spellcheck="false" readonly placeholder="stdout / stderr"></textarea>
      </div>
    </div>
    <div id="play-foot">
      <span id="play-status">ready</span>
    </div>
  </div>
</div>
</body>
</html>`;
}

function renderHtmlPlayground(ast, opt) {
    const title = (opt && opt.title) ? opt.title : 'NEPLg2 Tutorial';
    const description = (opt && opt.description)
        ? opt.description
        : 'NEPLg2 tutorial with interactive runnable examples.';
    const moduleJsPath = (opt && opt.moduleJsPath) ? String(opt.moduleJsPath) : './nepl-web.js';
    const body = renderBody(ast);
    return wrapHtmlPlayground(body, title, description, moduleJsPath);
}

module.exports = {
    renderHtmlPlayground,
};
