using System.Text.RegularExpressions;

namespace MDreader.Util;

/// <summary>
/// CommonMark fence-line matcher — port of linux fence.rs / macOS Fence.swift.
/// Matches only when the ENTIRE line is a fence opener/closer.
/// </summary>
public sealed record FenceMatch(string Indent, string Marker, string Tag, string Attrs);

public static class Fence
{
    // \w in Rust's regex crate is ASCII-only; .NET \w is Unicode by default, so
    // spell it out to keep the language-tag char class identical across ports.
    private static readonly Regex Re = new(
        @"^([ \t]{0,3})(`{3,}|~{3,})[ \t]*([0-9A-Za-z_-]+)?[ \t]*(\{.*\})?[ \t]*\z",
        RegexOptions.Compiled);

    /// <summary>Strips trailing Unicode whitespace (Swift trimTrailingWhitespace).</summary>
    public static string TrimTrailingWhitespace(string s) => s.TrimEnd();

    /// <summary>Returns the fence components iff the whole line is a fence line.</summary>
    public static FenceMatch? MatchLine(string line)
    {
        var m = Re.Match(line);
        if (!m.Success || m.Index != 0 || m.Length != line.Length) return null;
        return new FenceMatch(
            m.Groups[1].Value,
            m.Groups[2].Value,
            m.Groups[3].Value,
            m.Groups[4].Value);
    }
}
