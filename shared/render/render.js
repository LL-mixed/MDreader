// MDreader client-side renderer.
// The page (assets/render/index.html) is loaded with WebView.loadUrl, so this
// script and its sibling assets are all same-origin and load reliably. The
// markdown source and theme are provided at runtime by the Kotlin bridge
// window.mdreaderNative (see MarkdownView).
(function () {
  function render() {
    var native = window.mdreaderNative;
    var src = native ? native.getMarkdown() : '';
    var dark = native ? native.getDark() : false;
    document.body.className = dark ? 'dark' : 'light';

    var root = document.getElementById('content');
    root.innerHTML = renderMarkdown(src);

    if (window.katex) { renderMath(root); }
    renderMermaid(root);
    if (window.hljs) {
      document.querySelectorAll('pre code').forEach(function (block) {
        try { window.hljs.highlightElement(block); } catch (e) { /* ignore */ }
      });
    }
    if (native) { native.markRendered(); }
    // Outline: index headings (same DOM order marked produced) and report
    // them back to Kotlin, then (re)wire the scroll-spy that highlights the
    // active heading. Both run on every render so content/theme changes stay
    // in sync.
    indexHeadings(root);
    setupActiveObserver();
  }

  // Parses [src] with marked and restores SVGs that Kotlin's SvgGuard lifted
  // out (each top-level <svg>…</svg> was replaced by a \u0001{index}\u0002
  // placeholder before reaching here). marked follows CommonMark's HTML-block
  // rule that ends a block at the first blank line, which truncates large
  // SVGs mid-way; doing the lift in Kotlin keeps that logic JVM-tested and
  // fenced-code-aware (see SvgGuard). Mermaid SVG never reaches marked, so it
  // is unaffected.
  function renderMarkdown(src) {
    var html = window.marked ? window.marked.parse(src) : '<pre>' + src + '</pre>';
    var native = window.mdreaderNative;
    if (native && typeof native.getSvg === 'function') {
      html = html.replace(/\u0001(\d+)\u0002/g, function (_, i) {
        return native.getSvg(Number(i));
      });
    }
    return html;
  }

  window.MDreader = { render: render, scrollToHeading: scrollToHeading };
  document.addEventListener('DOMContentLoaded', render);

  // Walks text nodes under [node] and renders $...$ (inline) and $$...$$ (display) math.
  function renderMath(node) {
    var delims = [
      { left: '$$', right: '$$', display: true },
      { left: '$', right: '$', display: false }
    ];
    walk(node);

    function walk(n) {
      if (n.nodeType === 3) { processText(n); return; }
      if (n.nodeType !== 1) return;
      var tag = n.tagName;
      if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'CODE' || tag === 'PRE') return;
      var kids = [];
      for (var i = 0; i < n.childNodes.length; i++) kids.push(n.childNodes[i]);
      kids.forEach(walk);
    }

    function processText(textNode) {
      var text = textNode.nodeValue;
      if (text.indexOf('$') === -1) return;
      var frag = document.createDocumentFragment();
      var rest = text;
      var wrote = false;
      while (rest.length) {
        var bestIdx = -1, bestD = -1;
        for (var d = 0; d < delims.length; d++) {
          var idx = rest.indexOf(delims[d].left);
          if (idx >= 0 && (bestIdx === -1 || idx < bestIdx)) { bestIdx = idx; bestD = d; }
        }
        if (bestIdx === -1) { frag.appendChild(document.createTextNode(rest)); break; }
        var dl = delims[bestD];
        var contentStart = bestIdx + dl.left.length;
        var endIdx = rest.indexOf(dl.right, contentStart);
        if (endIdx === -1) {
          frag.appendChild(document.createTextNode(rest.slice(0, contentStart)));
          rest = rest.slice(contentStart);
          continue;
        }
        if (bestIdx > 0) frag.appendChild(document.createTextNode(rest.slice(0, bestIdx)));
        var math = rest.slice(contentStart, endIdx);
        try {
          var out = window.katex.renderToString(math, { displayMode: dl.display, throwOnError: false });
          var wrap = document.createElement(dl.display ? 'div' : 'span');
          wrap.className = 'math-' + (dl.display ? 'block' : 'inline');
          wrap.innerHTML = out;
          frag.appendChild(wrap);
        } catch (e) {
          frag.appendChild(document.createTextNode(rest.slice(bestIdx, endIdx + dl.right.length)));
        }
        wrote = true;
        rest = rest.slice(endIdx + dl.right.length);
      }
      if (wrote) textNode.parentNode.replaceChild(frag, textNode);
    }
  }

  // Replaces fenced mermaid blocks with rendered SVG diagrams. Uses the
  // Mermaid 11 API — mermaid.initialize() + async mermaid.render(id, code) —
  // which is the path that renders reliably on-device; the legacy
  // mermaid.init() from 9.x did not. Non-standard fence tags (```sequence,
  // ```gantt, …) are normalized to ```mermaid before reaching the WebView
  // (see MermaidFenceNormalizer), so only the `mermaid` tag is handled here.
  var mermaidSeq = 0;
  function renderMermaid(root) {
    var isDark = /(^|\s)dark(\s|$)/.test(document.body.className);
    var tasks = [];
    root.querySelectorAll('pre code.language-mermaid').forEach(function (code) {
      var pre = code.parentNode;
      if (!pre || pre.tagName !== 'PRE') return;
      var div = document.createElement('div');
      div.className = 'mermaid';
      pre.parentNode.replaceChild(div, pre);
      tasks.push({ div: div, code: code.textContent });
    });
    if (!tasks.length) return;
    if (window.mermaid) { renderMermaidDiagrams(tasks, isDark); return; }
    // mermaid.min.js assigns globalThis.mermaid at the tail of the bundle. In
    // the rare case render() runs before that assignment, retry once shortly
    // after so diagrams still render instead of being silently dropped.
    setTimeout(function () { if (window.mermaid) renderMermaidDiagrams(tasks, isDark); }, 300);
  }

  async function renderMermaidDiagrams(tasks, isDark) {
    try {
      window.mermaid.initialize({
        startOnLoad: false,
        theme: isDark ? 'dark' : 'default',
        securityLevel: 'loose',
        fontFamily: 'inherit',
        flowchart: { useMaxWidth: true, htmlLabels: true }
      });
    } catch (e) { /* ignore init error; per-diagram render surfaces real failures */ }
    for (var i = 0; i < tasks.length; i++) {
      var t = tasks[i];
      try {
        var id = 'mdr-mermaid-' + (++mermaidSeq);
        var result = await window.mermaid.render(id, t.code);
        t.div.innerHTML = result.svg;
      } catch (e) {
        t.div.innerHTML =
          '<div class="mermaid-error">⚠ Mermaid ' + escapeHtml(String((e && e.message) || e)) + '</div>';
      }
    }
  }

  function escapeHtml(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  // Outline (table of contents). Headings are read from the rendered DOM in
  // document order — the single source of truth for both the list Kotlin shows
  // and the scroll target — so the outline index and the DOM index always
  // agree. A Kotlin-side markdown parser would drift from marked's output on
  // Setext/HTML headings and misalign the index ("click A, jump to B").
  var HEADING_SELECTOR =
    '#content h1, #content h2, #content h3, #content h4, #content h5, #content h6';

  // Assigns sequential ids to every heading in document order and reports the
  // [{index, level, text}] list back to Kotlin. Runs after marked/Math/Mermaid
  // setup; Mermaid replaces pre>code with a div and never emits headings, so
  // the heading set is stable here.
  function indexHeadings(root) {
    var nodes = root.querySelectorAll('h1, h2, h3, h4, h5, h6');
    var outline = [];
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      el.id = 'mdr-h-' + i;
      outline.push({
        index: i,
        level: parseInt(el.tagName.substring(1), 10),
        text: (el.textContent || '').replace(/\s+/g, ' ').trim()
      });
    }
    var native = window.mdreaderNative;
    if (native && typeof native.onOutline === 'function') {
      native.onOutline(JSON.stringify(outline));
    }
  }

  function scrollToHeading(index) {
    var el = document.getElementById('mdr-h-' + index);
    if (!el) return;
    try { el.scrollIntoView({ behavior: 'auto', block: 'start' }); }
    catch (e) { el.scrollIntoView(); }
    pickActiveHeading();
  }

  // Scroll-spy: highlights the heading of the section currently in view.
  // IntersectionObserver is used only as a "a heading crossed the top slice"
  // signal; pickActiveHeading recomputes the active index by geometry, which
  // is simpler and robust when several headings are visible at once. lastActive
  // dedupes so Kotlin is notified only on change (not every intersection tick).
  var activeObserver = null;
  var lastActive = -1;
  var ACTIVE_CUTOFF_PX = 96; // a heading counts as "current" once near the top

  function pickActiveHeading() {
    var nodes = document.querySelectorAll(HEADING_SELECTOR);
    var active = -1;
    for (var i = 0; i < nodes.length; i++) {
      if (nodes[i].getBoundingClientRect().top <= ACTIVE_CUTOFF_PX) active = i;
      else break;
    }
    if (active === -1 && nodes.length) active = 0; // above the first heading
    if (active !== lastActive) {
      lastActive = active;
      var native = window.mdreaderNative;
      if (native && typeof native.onActiveHeading === 'function') {
        native.onActiveHeading(active);
      }
    }
  }

  function setupActiveObserver() {
    if (activeObserver) { activeObserver.disconnect(); activeObserver = null; }
    lastActive = -1;
    var nodes = document.querySelectorAll(HEADING_SELECTOR);
    if (!nodes.length) return;
    pickActiveHeading();
    if (!('IntersectionObserver' in window)) return;
    activeObserver = new IntersectionObserver(function () { pickActiveHeading(); }, {
      rootMargin: '0px 0px -80% 0px',
      threshold: 0
    });
    for (var i = 0; i < nodes.length; i++) activeObserver.observe(nodes[i]);
  }
})();
