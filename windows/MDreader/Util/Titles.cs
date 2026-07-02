namespace MDreader.Util;

/// <summary>
/// Derives a display title from a file path, stripping a trailing markdown
/// extension. Port of linux titles.rs / macOS Titles.swift.
/// </summary>
public static class Titles
{
    private static readonly HashSet<string> MarkdownExts = new(StringComparer.OrdinalIgnoreCase)
    {
        "md", "markdown", "mdown", "mkd", "mkdown",
    };

    public static string FromPath(string path)
    {
        if (path.Length == 0) return "";
        int sep = path.LastIndexOfAny(new[] { '/', '\\' });
        int nameStart = sep >= 0 ? sep + 1 : 0;
        string name = path.Substring(nameStart);
        int dot = name.LastIndexOf('.');
        if (dot < 0) return name;
        if (dot == 0) return name; // hidden like ".md"
        string ext = name.Substring(dot + 1).ToLowerInvariant();
        return MarkdownExts.Contains(ext) ? name.Substring(0, dot) : name;
    }
}
