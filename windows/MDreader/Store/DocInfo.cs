namespace MDreader.Store;

/// <summary>
/// An immutable value snapshot of a cached document, decoupled from the SQLite
/// row so the UI holds plain data. Port of linux doc_info.rs / macOS DocInfo.swift.
/// </summary>
public sealed record DocInfo(
    Guid Id,
    string Title,
    string ContentHash,
    string? SourceUri,
    long OpenedAt,
    bool Favorite,
    long CharCount);
