using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using MDreader.Render;
using MDreader.Store;
using MDreader.Util;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.Web.WebView2.Core;

namespace MDreader;

/// <summary>
/// Main reader window: split layout with a library/outline sidebar and a WebView2 reader.
/// Covers WM3-WM6 — file open + cache, content management, outline, zoom, session restore,
/// external editor, PDF export, in-page drop.
/// </summary>
public sealed partial class MainWindow : Window
{
    private const string DropScript =
        "(function(){" +
        "  document.addEventListener('dragover', function(e){ e.preventDefault(); });" +
        "  document.addEventListener('drop', function(e){" +
        "    e.preventDefault();" +
        "    var f = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];" +
        "    if (!f) return;" +
        "    if (!/\\.(md|markdown|mdown|mkd|mkdown)$/i.test(f.name || '')) return;" +
        "    var reader = new FileReader();" +
        "    reader.onload = function(){" +
        "      window.chrome.webview.postMessage({event:'dropFile', name:f.name, text:reader.result});" +
        "    };" +
        "    reader.readAsText(f);" +
        "  });" +
        "})();";

    private readonly DocRepository _repo;
    private readonly SessionStore _session;
    private readonly ZoomStore _zoom;
    private readonly SettingsStore _settings;

    private Guid? _currentId;
    private string? _currentHash;
    private string? _currentSourceUri;
    private string? _currentTitle;
    private bool _favFilter;
    private string? _searchQuery;
    private bool _loaded;

    public MainWindow(string? filePath)
    {
        InitializeComponent();
        Title = "MDreader";
        var dir = DataDir;
        Directory.CreateDirectory(dir);
        _repo = DocRepository.Open(dir);
        _session = new SessionStore(dir);
        _zoom = new ZoomStore(dir);
        _settings = new SettingsStore(dir);
        LoadLibrary();
        _ = InitAsync(filePath);
    }

    private static string DataDir => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "MDreader");

    private async Task InitAsync(string? filePath)
    {
        await EnsureWebViewAsync();
        if (filePath != null && File.Exists(filePath))
        {
            await OpenFileAsync(filePath);
        }
        else if (_session.LastDocID is Guid id && _repo.LoadContent(id) != null)
        {
            await OpenDocAsync(id);
        }
        else
        {
            await OpenSampleAsync();
        }
    }

    private async Task OpenFileAsync(string path)
    {
        var md = await File.ReadAllTextAsync(path);
        var title = Titles.FromPath(path);
        var id = _repo.Cache(title, md, path);
        await OpenDocAsync(id);
    }

    private async Task OpenDocAsync(Guid id)
    {
        var md = _repo.LoadContent(id);
        if (md == null) return;
        var doc = _repo.All().FirstOrDefault(d => d.Id == id);
        if (doc == null) return;
        _currentId = id;
        _currentHash = doc.ContentHash;
        _currentSourceUri = doc.SourceUri;
        _currentTitle = doc.Title;
        Title = doc.Title + " — MDreader";
        _session.SetLastDocID(id);
        await RenderDocumentAsync(md, doc.SourceUri != null ? Path.GetDirectoryName(doc.SourceUri) : null);
        ApplyZoom();
    }

    private async Task OpenSampleAsync()
    {
        var samplePath = Path.Combine(AppContext.BaseDirectory, "sample.md");
        var md = File.Exists(samplePath) ? await File.ReadAllTextAsync(samplePath) : "# MDreader";
        _currentHash = ContentHash.Sha256Hex(md);
        _currentTitle = "MDreader";
        _currentSourceUri = null;
        await RenderDocumentAsync(md, AppContext.BaseDirectory);
        ApplyZoom();
    }

    private async Task EnsureWebViewAsync()
    {
        if (WebView.CoreWebView2 != null) return;
        await WebView.EnsureCoreWebView2Async();
        var core = WebView.CoreWebView2;
        var renderDir = Path.Combine(AppContext.BaseDirectory, "render");
        core.SetVirtualHostNameToFolderMapping(
            "app.local", renderDir, CoreWebView2HostResourceAccessKind.Allow);
        core.WebMessageReceived += OnWebMessageReceived;
        await core.AddScriptToExecuteOnDocumentCreatedAsync(DropScript);
    }

    private async Task RenderDocumentAsync(string md, string? baseDir)
    {
        await EnsureWebViewAsync();
        var core = WebView.CoreWebView2;
        var payload = PayloadBuilder.BuildJson(md, dark: false, baseDir);
        if (!_loaded)
        {
            await core.AddScriptToExecuteOnDocumentCreatedAsync(BridgeShim.Build(payload));
            core.Navigate("https://app.local/index.html");
            _loaded = true;
        }
        else
        {
            await core.ExecuteScriptAsync(
                "window.__mdrPayload = " + payload + "; if(window.MDreader){window.MDreader.render();}");
        }
    }

    private void ApplyZoom()
    {
        if (WebView.CoreWebView2 == null || _currentHash == null) return;
        var z = _zoom.ZoomFor(_currentHash) ?? 1.0;
        WebView.CoreWebView2.ZoomFactor = z;
        ZoomLabel.Text = Math.Round(z * 100) + "%";
    }

    private void LoadLibrary()
    {
        var docs = _repo.All();
        var groups = LibraryModel.Build(docs, _favFilter, _searchQuery);
        var cvs = new CollectionViewSource { IsSourceGrouped = true, Source = groups };
        DocList.ItemsSource = cvs.View;
    }

    private void OnTabToggle(object sender, RoutedEventArgs e)
    {
        if (sender == LibTab && LibTab.IsChecked == true) OutlineTab.IsChecked = false;
        else if (sender == OutlineTab && OutlineTab.IsChecked == true) LibTab.IsChecked = false;
        if (LibTab.IsChecked != true && OutlineTab.IsChecked != true) LibTab.IsChecked = true;
        DocList.Visibility = LibTab.IsChecked == true ? Visibility.Visible : Visibility.Collapsed;
        OutlineList.Visibility = OutlineTab.IsChecked == true ? Visibility.Visible : Visibility.Collapsed;
    }

    private void OnFavFilter(object sender, RoutedEventArgs e)
    {
        _favFilter = FavFilter.IsChecked == true;
        LoadLibrary();
    }

    private void OnSearch(object sender, TextChangedEventArgs e)
    {
        _searchQuery = string.IsNullOrWhiteSpace(SearchBox.Text) ? null : SearchBox.Text;
        LoadLibrary();
    }

    private async void OnDocClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is DocRow row) await OpenDocAsync(row.Id);
    }

    private void OnDocRightTapped(object sender, RightTappedRoutedEventArgs e)
    {
        var fe = e.OriginalSource as FrameworkElement;
        if (fe?.DataContext is not DocRow row) return;
        var doc = _repo.All().FirstOrDefault(d => d.Id == row.Id);
        if (doc == null) return;

        var flyout = new MenuFlyout();
        var fav = new MenuFlyoutItem { Text = doc.Favorite ? "取消收藏" : "收藏" };
        fav.Click += (_, _) => { _repo.SetFavorite(doc.Id, !doc.Favorite); LoadLibrary(); };
        var refresh = new MenuFlyoutItem { Text = "从源刷新" };
        refresh.Click += async (_, _) => { if (_repo.RefreshFromSource(doc.Id)) { await OpenDocAsync(doc.Id); LoadLibrary(); } };
        var newWin = new MenuFlyoutItem { Text = "新窗口打开" };
        newWin.Click += (_, _) => { var w = new MainWindow(null); w.Activate(); };
        var del = new MenuFlyoutItem { Text = "删除" };
        del.Click += (_, _) => { _repo.Delete(doc.Id); if (_currentId == doc.Id) _ = OpenSampleAsync(); LoadLibrary(); };

        flyout.Items.Add(fav);
        flyout.Items.Add(refresh);
        flyout.Items.Add(new MenuFlyoutSeparator());
        flyout.Items.Add(newWin);
        flyout.Items.Add(del);
        flyout.ShowAt(fe, e.GetPosition(fe));
    }

    private void OnOutlineClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is OutlineRow r && WebView.CoreWebView2 != null)
        {
            _ = WebView.CoreWebView2.ExecuteScriptAsync(
                "if(window.MDreader){window.MDreader.scrollToHeading(" + r.Index + ");}");
        }
    }

    private void OnZoomIn(object sender, RoutedEventArgs e) => BumpZoom(+0.1);
    private void OnZoomOut(object sender, RoutedEventArgs e) => BumpZoom(-0.1);
    private void OnZoomReset(object sender, RoutedEventArgs e) => SetZoom(1.0);

    private void BumpZoom(double delta)
    {
        if (WebView.CoreWebView2 == null) return;
        SetZoom(Math.Clamp(WebView.CoreWebView2.ZoomFactor + delta, 0.3, 3.0));
    }

    private void SetZoom(double z)
    {
        if (WebView.CoreWebView2 == null) return;
        WebView.CoreWebView2.ZoomFactor = z;
        ZoomLabel.Text = Math.Round(z * 100) + "%";
        if (_currentHash != null) _zoom.SetZoom(z, _currentHash);
    }

    private async void OnRefresh(object sender, RoutedEventArgs e)
    {
        if (_currentId is Guid id && _repo.RefreshFromSource(id))
        {
            await OpenDocAsync(id);
            LoadLibrary();
        }
    }

    private void OnOpenEditor(object sender, RoutedEventArgs e)
    {
        var src = _currentSourceUri;
        if (string.IsNullOrEmpty(src)) return;
        var cmd = _settings.EditorCommand;
        try
        {
            if (string.IsNullOrWhiteSpace(cmd))
            {
                System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo(src) { UseShellExecute = true });
                return;
            }
            var parts = cmd.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
            var exe = parts[0];
            var argList = parts.Skip(1).Append("\"" + src + "\"");
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo(exe, string.Join(" ", argList))
            {
                UseShellExecute = false,
            });
        }
        catch
        {
            // Editor not found / launch failed — ignore.
        }
    }

    private async void OnExportPdf(object sender, RoutedEventArgs e)
    {
        if (WebView.CoreWebView2 == null) return;
        var name = string.IsNullOrWhiteSpace(_currentTitle) ? "mdreader" : _currentTitle;
        foreach (var c in Path.GetInvalidFileNameChars()) name = name.Replace(c, '_');
        var path = Path.Combine(Path.GetTempPath(), name + ".pdf");
        try
        {
            await WebView.CoreWebView2.PrintToPdfAsync(path, new CoreWebView2PrintSettings());
            Title = (_currentTitle ?? "MDreader") + " — 已导出 " + path;
        }
        catch
        {
            // PrintToPdf failed — ignore.
        }
    }

    private async void OnAbout(object sender, RoutedEventArgs e)
    {
        var dlg = new ContentDialog
        {
            Title = "MDreader",
            Content = "Markdown 阅读器  ·  0.1.0\nAndroid · macOS · Linux · Windows",
            CloseButtonText = "关闭",
            XamlRoot = WebView.XamlRoot,
        };
        await dlg.ShowAsync();
    }

    private void OnWebMessageReceived(object? sender, CoreWebView2WebMessageReceivedEventArgs e)
    {
        try
        {
            using var doc = JsonDocument.Parse(e.WebMessageAsJson);
            if (!doc.RootElement.TryGetProperty("event", out var ev)) return;
            switch (ev.GetString())
            {
                case "dropFile":
                    {
                        var name = doc.RootElement.TryGetProperty("name", out var n) ? n.GetString() ?? "" : "";
                        var text = doc.RootElement.TryGetProperty("text", out var t) ? t.GetString() ?? "" : "";
                        var title = string.IsNullOrEmpty(name) ? DocRepository.DefaultTitle : Path.GetFileNameWithoutExtension(name);
                        var id = _repo.Cache(title, text, null);
                        _ = OpenDocAsync(id).ContinueWith(_ => LoadLibrary(), TaskScheduler.FromCurrentSynchronizationContext());
                        break;
                    }
                case "onOutline":
                    {
                        var json = doc.RootElement.TryGetProperty("json", out var j) ? j.GetString() ?? "[]" : "[]";
                        var items = Outline.Parse(json);
                        var rows = items?.Select(o =>
                            new OutlineRow(o.Index, o.Text,
                                new Thickness(Math.Max(0, (int)o.Level - 1) * 12, 0, 0, 0))).ToList();
                        OutlineList.ItemsSource = rows;
                        break;
                    }
                case "onActiveHeading":
                    // WM5: hook up scroll-spy highlight here if desired.
                    break;
            }
        }
        catch
        {
            // Ignore malformed bridge messages.
        }
    }
}
