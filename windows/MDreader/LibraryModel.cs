using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using MDreader.Store;
using MDreader.Util;
using Microsoft.UI.Xaml;

namespace MDreader;

public sealed record DocRow(Guid Id, string Title, string Subtitle);

public sealed record OutlineRow(int Index, string Text, Thickness Indent);

/// <summary>
/// A date-bucketed section of the library list (CollectionViewSource group).
/// </summary>
public sealed class DocGroup : ObservableCollection<DocRow>
{
    public string BucketTitle { get; }
    public DocGroup(string bucketTitle) => BucketTitle = bucketTitle;
}

/// <summary>
/// Builds date-bucketed groups for the library list. Port of the grouping logic
/// behind macOS <c>LibraryView</c> / linux <c>app.rs</c> sidebar.
/// </summary>
public static class LibraryModel
{
    public static List<DocGroup> Build(IReadOnlyList<DocInfo> docs, bool favoritesOnly, string? query)
    {
        IEnumerable<DocInfo> filtered = favoritesOnly ? docs.Where(d => d.Favorite) : docs;
        if (!string.IsNullOrEmpty(query))
        {
            var q = query.ToLowerInvariant();
            filtered = filtered.Where(d => d.Title.ToLowerInvariant().Contains(q, StringComparison.Ordinal));
        }
        var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        var result = new List<DocGroup>();
        foreach (var b in DateBuckets.All)
        {
            var rows = filtered
                .Where(d => DateBuckets.Bucket(d.OpenedAt, now) == b)
                .Select(d => new DocRow(d.Id, d.Title, DateBuckets.Format(d.OpenedAt)))
                .ToList();
            if (rows.Count == 0) continue;
            var g = new DocGroup(b.Title());
            foreach (var r in rows) g.Add(r);
            result.Add(g);
        }
        return result;
    }
}
