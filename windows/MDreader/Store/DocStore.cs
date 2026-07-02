using System.IO;

namespace MDreader.Store;

/// <summary>
/// Filesystem body storage: one <c>&lt;uuid&gt;.md</c> per doc.
/// Port of linux doc_store.rs / macOS DocStore.swift.
/// </summary>
public static class DocStore
{
    public static string FilePath(string docsDir, Guid id) =>
        Path.Combine(docsDir, id + ".md");

    public static void Write(string docsDir, Guid id, string markdown)
    {
        Directory.CreateDirectory(docsDir);
        File.WriteAllText(FilePath(docsDir, id), markdown);
    }

    public static string? Read(string docsDir, Guid id)
    {
        var p = FilePath(docsDir, id);
        return File.Exists(p) ? File.ReadAllText(p) : null;
    }

    public static void Delete(string docsDir, Guid id)
    {
        var p = FilePath(docsDir, id);
        if (File.Exists(p)) File.Delete(p);
    }
}
