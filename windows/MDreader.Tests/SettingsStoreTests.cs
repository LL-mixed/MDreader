using System.IO;
using MDreader.Store;
using Xunit;

namespace MDreader.Tests;

public class SettingsStoreTests
{
    private static string Tmp()
    {
        var dir = Path.Combine(Path.GetTempPath(), "mdreader-settings-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        return dir;
    }

    [Fact]
    public void MissingFileDefaultsToEmptyEditor()
    {
        var dir = Tmp();
        try
        {
            var s = new SettingsStore(dir);
            Assert.Equal("", s.EditorCommand);
        }
        finally { TryDelete(dir); }
    }

    [Fact]
    public void RoundTripPersistsEditorCommand()
    {
        var dir = Tmp();
        try
        {
            var s = new SettingsStore(dir);
            s.SetEditorCommand("Typora");
            Assert.Equal("Typora", s.EditorCommand);
            var s2 = new SettingsStore(dir);
            Assert.Equal("Typora", s2.EditorCommand);
        }
        finally { TryDelete(dir); }
    }

    [Fact]
    public void TolerantOfCorruptJson()
    {
        var dir = Tmp();
        try
        {
            File.WriteAllText(Path.Combine(dir, "config.json"), "{not valid");
            var s = new SettingsStore(dir);
            Assert.Equal("", s.EditorCommand);
        }
        finally { TryDelete(dir); }
    }

    [Fact]
    public void UnknownKeysAreIgnored()
    {
        var dir = Tmp();
        try
        {
            File.WriteAllText(Path.Combine(dir, "config.json"), "{\"editorCommand\":\"Code\",\"futureKey\":7}");
            var s = new SettingsStore(dir);
            Assert.Equal("Code", s.EditorCommand);
        }
        finally { TryDelete(dir); }
    }

    private static void TryDelete(string dir)
    {
        try { Directory.Delete(dir, true); } catch { }
    }
}
