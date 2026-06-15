// MDreader client-side renderer.
// Reads the markdown source injected by MarkdownHtmlBuilder as window.MD_SOURCE,
// parses it with marked, renders math via KaTeX, diagrams via Mermaid, and
// highlights code with highlight.js.
(function () {
  var root = document.getElementById('content');
  var src = window.MD_SOURCE || '';
  var html = window.marked ? window.marked.parse(src) : '<pre>' + src + '</pre>';
  root.innerHTML = html;

  if (window.katex) {
    renderMath(root);
  }
  if (window.mermaid) {
    renderMermaid(root);
  }
  if (window.hljs) {
    document.querySelectorAll('pre code').forEach(function (block) {
      try { window.hljs.highlightElement(block); } catch (e) { /* ignore */ }
    });
  }

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
          // Prefer $$ over $ when both start at the same index (delims[0] is $$).
          if (idx >= 0 && (bestIdx === -1 || idx < bestIdx)) { bestIdx = idx; bestD = d; }
        }
        if (bestIdx === -1) { frag.appendChild(document.createTextNode(rest)); break; }
        var dl = delims[bestD];
        var contentStart = bestIdx + dl.left.length;
        var endIdx = rest.indexOf(dl.right, contentStart);
        if (endIdx === -1) {
          // No closing delimiter: emit up to the opener literally and keep scanning.
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

  // Replaces ```mermaid fenced blocks with rendered SVG diagrams.
  function renderMermaid(root) {
    var isDark = /(^|\s)dark(\s|$)/.test(document.body.className);
    var blocks = root.querySelectorAll('pre code.language-mermaid');
    if (!blocks.length) return;
    blocks.forEach(function (code, i) {
      var pre = code.parentNode;
      if (!pre || pre.tagName !== 'PRE') return;
      var div = document.createElement('div');
      div.className = 'mermaid';
      div.id = 'mdr-mermaid-' + i;
      div.textContent = code.textContent;
      pre.parentNode.replaceChild(div, pre);
    });
    try {
      window.mermaid.init({
        startOnLoad: false,
        theme: isDark ? 'dark' : 'default',
        securityLevel: 'loose',
        fontFamily: 'inherit'
      }, root.querySelectorAll('.mermaid'));
    } catch (e) { /* ignore */ }
  }
})();
