using System.Text.Json;
using MDreader.Util;

namespace MDreader.Render;

/// <summary>
/// Builds the <c>window.__mdrPayload</c> JSON object <c>{ md, dark, svgs }</c>
/// consumed by the bridge shim and by <c>render.js</c>.
/// </summary>
/// <remarks>
/// The native preprocessing pipeline (resolve_images -> mermaid_fence ->
/// svg_guard) runs here, the C# port of
/// <c>linux/src/render/preprocess.rs::build_payload</c>.
/// </remarks>
public static class PayloadBuilder
{
    /// <summary>
    /// Run the full preprocessing pipeline and serialize the payload.
    /// Mirrors linux <c>build_payload</c> / macOS <c>buildPayload</c>.
    /// </summary>
    public static string BuildJson(string markdown, bool dark, string? baseDir)
    {
        var resolved = Preprocess.ResolveImages(markdown, baseDir);
        var normalized = MermaidFenceNormalizer.Normalize(resolved);
        var guarded = SvgGuard.Protect(normalized);
        return BuildJson(guarded.Markdown, dark, guarded.Svgs);
    }
    public static string BuildJson(string markdown, bool dark, IReadOnlyList<string> svgs)
    {
        using var ms = new MemoryStream();
        using (var writer = new Utf8JsonWriter(ms))
        {
            writer.WriteStartObject();
            writer.WriteString("md", markdown);
            writer.WriteBoolean("dark", dark);
            writer.WriteStartArray("svgs");
            foreach (var svg in svgs)
            {
                writer.WriteStringValue(svg);
            }
            writer.WriteEndArray();
            writer.WriteEndObject();
        }
        return System.Text.Encoding.UTF8.GetString(ms.ToArray());
    }

    public static string BuildJson(string markdown, bool dark)
        => BuildJson(markdown, dark, Array.Empty<string>());
}
