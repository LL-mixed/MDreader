using System.Text;
using System.Text.RegularExpressions;

namespace MDreader.Util;

/// <summary>
/// Guarded markdown: the rewritten text plus the lifted SVGs.
/// </summary>
public sealed record Guarded(string Markdown, IReadOnlyList<string> Svgs);

/// <summary>
/// SvgGuard — port of linux svg_guard.rs / macOS SvgGuard.swift.
/// Lifts top-level &lt;svg&gt;…&lt;/svg&gt; blocks out of markdown into
/// <c>\u0001{index}\u0002</c> placeholders (restored by JS via getSvg), because
/// marked truncates large inline SVGs at blank lines. Fence-aware: SVGs inside
/// ```/~~~ code blocks are left untouched.
/// </summary>
public static class SvgGuard
{
    public const char Marker = '\u0001';
    public const char End = '\u0002';
    private static readonly Regex SvgRe = new(@"<svg\b[\s\S]*?</svg>", RegexOptions.Compiled);

    public static string Placeholder(int index) => $"{Marker}{index}{End}";

    public static Guarded Protect(string markdown)
    {
        if (!markdown.Contains("<svg"))
            return new Guarded(markdown, Array.Empty<string>());

        var svgs = new List<string>();
        var lines = markdown.Split('\n');
        var sb = new StringBuilder(markdown.Length);
        int i = 0;
        bool inFence = false;
        string fenceMarker = "";
        while (i < lines.Length)
        {
            var line = lines[i];
            var fm = Fence.MatchLine(Fence.TrimTrailingWhitespace(line));
            if (fm != null)
            {
                if (!inFence)
                {
                    inFence = true;
                    fenceMarker = fm.Marker;
                }
                else if (fm.Marker.Length > 0 && fenceMarker.Length > 0
                         && fm.Marker[0] == fenceMarker[0]
                         && fm.Marker.Length >= fenceMarker.Length)
                {
                    inFence = false;
                    fenceMarker = "";
                }
                sb.Append(line);
                sb.Append('\n');
                i++;
                continue;
            }
            if (inFence)
            {
                sb.Append(line);
                sb.Append('\n');
                i++;
                continue;
            }
            if (line.Contains("<svg"))
            {
                var buf = new StringBuilder(line);
                int j = i;
                if (!line.Contains("</svg>"))
                {
                    j = i + 1;
                    while (j < lines.Length)
                    {
                        buf.Append('\n');
                        buf.Append(lines[j]);
                        if (lines[j].Contains("</svg>")) break;
                        j++;
                    }
                }
                var replaced = ExtractSvgs(buf.ToString(), svgs);
                sb.Append(replaced);
                sb.Append('\n');
                i = j + 1;
                continue;
            }
            sb.Append(line);
            sb.Append('\n');
            i++;
        }
        if (sb.Length > 0 && sb[sb.Length - 1] == '\n') sb.Length -= 1;
        return new Guarded(sb.ToString(), svgs);
    }

    private static string ExtractSvgs(string text, List<string> svgs)
    {
        var sb = new StringBuilder(text.Length);
        int cursor = 0;
        foreach (Match m in SvgRe.Matches(text))
        {
            sb.Append(text, cursor, m.Index - cursor);
            svgs.Add(m.Value);
            sb.Append(Placeholder(svgs.Count - 1));
            cursor = m.Index + m.Length;
        }
        sb.Append(text, cursor, text.Length - cursor);
        return sb.ToString();
    }
}
