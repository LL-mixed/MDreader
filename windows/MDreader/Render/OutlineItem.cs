using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace MDreader.Render;

/// <summary>
/// One heading in the document outline: decode of render.js's onOutline payload
/// <c>[{index, level, text}]</c>. Port of linux outline.rs / macOS OutlineItem.swift.
/// </summary>
public sealed record OutlineItem(
    [property: JsonPropertyName("index")] int Index,
    [property: JsonPropertyName("level")] uint Level,
    [property: JsonPropertyName("text")] string Text);

public static class Outline
{
    /// <summary>Decode the JSON string render.js posts via onOutline; null on bad JSON.</summary>
    public static List<OutlineItem>? Parse(string json)
    {
        try
        {
            return JsonSerializer.Deserialize<List<OutlineItem>>(json);
        }
        catch
        {
            return null;
        }
    }
}
