using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace MDreader.Store;

/// <summary>
/// User-editable preferences. <c>editorCommand</c> mirrors macOS's Codable key name.
/// Port of linux settings_store.rs / macOS Settings.swift.
/// </summary>
public sealed record AppSettings
{
    [JsonPropertyName("editorCommand")]
    public string EditorCommand { get; set; } = "";
}

/// <summary>
/// Persists user preferences as <c>config.json</c>.
/// </summary>
public sealed class SettingsStore
{
    private readonly string _path;

    public AppSettings Settings { get; private set; }

    public SettingsStore(string dir)
    {
        _path = Path.Combine(dir, "config.json");
        try
        {
            Settings = JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(_path))
                       ?? new AppSettings();
        }
        catch
        {
            Settings = new AppSettings();
        }
    }

    public string EditorCommand => Settings.EditorCommand;

    public void SetEditorCommand(string command)
    {
        Settings = Settings with { EditorCommand = command };
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_path)!);
            File.WriteAllText(_path, JsonSerializer.Serialize(Settings));
        }
        catch
        {
            // Best-effort persistence.
        }
    }
}
