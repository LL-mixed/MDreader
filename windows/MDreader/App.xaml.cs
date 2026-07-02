using System.IO;
using Microsoft.UI.Xaml;

namespace MDreader;

/// <summary>
/// Application entry. Unpackaged (WindowsPackageType=None): the SDK generates a
/// suitable entry point and OnLaunched fires on startup.
/// </summary>
public partial class App : Application
{
    private MainWindow? _mainWindow;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Unpackaged: GetCommandLineArgs()[0] is the exe; [1] is the first argument —
        // the .md path when opened via the registered ProgId / double-click, or when
        // launched as `MDreader.exe path\to\file.md`.
        var argv = Environment.GetCommandLineArgs();
        string? fileArg = (argv.Length > 1 && File.Exists(argv[1])) ? argv[1] : null;
        _mainWindow = new MainWindow(fileArg);
        _mainWindow.Activate();
    }
}
