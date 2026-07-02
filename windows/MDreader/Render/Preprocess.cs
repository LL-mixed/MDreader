using System.IO;
using System.Text;
using System.Text.RegularExpressions;

namespace MDreader.Render;

/// <summary>
/// Image resolution — port of linux preprocess.rs::resolve_images /
/// macOS MarkdownWebView.resolveImages. Rewrites relative <c>![alt](rel)</c>
/// URLs against the file's directory: leaves http(s)://, /abs, file:, # untouched;
/// inlines <c>.svg</c> as raw SVG text (SvgGuard then lifts it); base64-encodes
/// raster types into data: URIs. Runs before the markdown reaches JS.
/// </summary>
public static class Preprocess
{
    private static readonly Regex ImgRe = new(@"(!\[[^\]]*\]\()([^)]+)(\))", RegexOptions.Compiled);

    public static string ResolveImages(string markdown, string? baseDir)
    {
        if (baseDir == null) return markdown;

        var sb = new StringBuilder(markdown.Length);
        int last = 0;
        foreach (Match m in ImgRe.Matches(markdown))
        {
            sb.Append(markdown, last, m.Index - last);
            string g1 = m.Groups[1].Value;
            string original = m.Groups[2].Value;
            string g3 = m.Groups[3].Value;

            // src = original up to the first space (Swift firstIndex(of: " ")).
            int sp = original.IndexOf(' ');
            string src = sp >= 0 ? original.Substring(0, sp) : original;

            if (src.StartsWith("http://") || src.StartsWith("https://")
                || src.StartsWith("/") || src.StartsWith("file:") || src.StartsWith("#"))
            {
                sb.Append(g1);
                sb.Append(original);
                sb.Append(g3);
            }
            else
            {
                var abs = Path.Combine(baseDir, src);
                var ext = Path.GetExtension(src).TrimStart('.').ToLowerInvariant();
                if (ext == "svg")
                {
                    try
                    {
                        var svg = File.ReadAllText(abs);
                        sb.Append("\n\n");
                        sb.Append(svg);
                        sb.Append("\n\n");
                    }
                    catch
                    {
                        sb.Append(g1);
                        sb.Append(original);
                        sb.Append(g3);
                    }
                }
                else
                {
                    var mime = ext switch
                    {
                        "png" => "image/png",
                        "jpg" or "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "webp" => "image/webp",
                        _ => "application/octet-stream",
                    };
                    try
                    {
                        var data = File.ReadAllBytes(abs);
                        var b64 = Convert.ToBase64String(data);
                        sb.Append(g1);
                        sb.Append($"data:{mime};base64,{b64}");
                        sb.Append(g3);
                    }
                    catch
                    {
                        sb.Append(g1);
                        sb.Append(original);
                        sb.Append(g3);
                    }
                }
            }
            last = m.Index + m.Length;
        }
        sb.Append(markdown, last, markdown.Length - last);
        return sb.ToString();
    }
}
