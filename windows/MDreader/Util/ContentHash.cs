using System.Security.Cryptography;
using System.Text;

namespace MDreader.Util;

/// <summary>
/// SHA-256 of UTF-8 bytes as lowercase hex. Port of linux content_hash.rs /
/// macOS ContentHash.swift. Used by the cache layer for content dedup.
/// </summary>
public static class ContentHash
{
    public static string Sha256Hex(string text)
    {
        var digest = SHA256.HashData(Encoding.UTF8.GetBytes(text));
        var sb = new StringBuilder(64);
        foreach (var b in digest)
        {
            sb.Append(b.ToString("x2"));
        }
        return sb.ToString();
    }
}
