import AppKit
import SwiftUI

struct AboutView: View {
    private let info = BuildInfo.current

    private var version: String {
        let v = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? ""
        let b = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? ""
        return "v\(v) (\(b))"
    }

    var body: some View {
        VStack(spacing: 12) {
            Image(nsImage: NSApp.applicationIconImage)
                .resizable()
                .frame(width: 96, height: 96)
            Text("MDreader").font(.title2).bold()
            Text(version)
                .foregroundStyle(.secondary)
                .font(.callout)
            Divider()
            HStack {
                VStack(alignment: .leading, spacing: 6) {
                    Label(info.author, systemImage: "person")
                    Label(info.buildTime, systemImage: "clock")
                    Label(info.gitHash, systemImage: "arrow.triangle.branch")
                }
                .font(.callout)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(24)
        .frame(width: 340)
    }
}

enum AboutWindow {
    private static var window: NSWindow?

    static func show() {
        NSApp.activate(ignoringOtherApps: true)
        if window == nil {
            let w = NSWindow(contentViewController: NSHostingController(rootView: AboutView()))
            w.styleMask = [.titled, .closable]
            w.titleVisibility = .hidden
            w.titlebarAppearsTransparent = true
            w.isReleasedWhenClosed = false
            window = w
        }
        window?.center()
        window?.makeKeyAndOrderFront(nil)
    }
}
