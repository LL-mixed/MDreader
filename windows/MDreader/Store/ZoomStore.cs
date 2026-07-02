using System.Collections.Generic;
using System.IO;
using System.Text.Json;

namespace MDreader.Store;

/// <summary>
/// Per-content-hash zoom map, persisted as <c>zoom.json</c>.
/// Port of linux zoom_store.rs / macOS ZoomStore.swift.
/// </summary>
public sealed class ZoomStore
{
    private readonly string _path;
    private readonly Dictionary<string, double> _map;

    public ZoomStore(string dir)
    {
        _path = Path.Combine(dir, "zoom.json");
        try
        {
            _map = JsonSerializer.Deserialize<Dictionary<string, double>>(File.ReadAllText(_path))
                   ?? new Dictionary<string, double>();
        }
        catch
        {
            _map = new Dictionary<string, double>();
        }
    }

    public double? ZoomFor(string hash) => _map.TryGetValue(hash, out var z) ? z : null;

    public void SetZoom(double zoom, string hash)
    {
        _map[hash] = zoom;
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_path)!);
            File.WriteAllText(_path, JsonSerializer.Serialize(_map));
        }
        catch
        {
            // Best-effort persistence.
        }
    }
}
