import AppKit
import SwiftUI

/// Forces a newly opened document window to appear as a tab of the current window.
///
/// SwiftUI's `openWindow` always creates an independent window on macOS; whether a
/// shared `tabbingIdentifier` + `tabbingMode = .preferred` produces a tab depends on
/// the system "Prefer tabs when opening documents" setting, whose default ("Manually")
/// yields a plain window. Here we observe the incoming window and call
/// `NSWindow.addTabbedWindow(_:ordered:)` so "open in new tab" behaves as a tab
/// regardless of that system setting.
enum WindowTabber {
    private static var pending: PendingTab?

    static func openDocAsTab(id: UUID, using openWindow: OpenWindowAction) {
        let regular = NSApp.windows.filter { !$0.isKind(of: NSPanel.self) }
        guard let source = NSApp.keyWindow ?? NSApp.mainWindow ?? regular.first else {
            openWindow(value: id)
            return
        }
        let existing = Set(NSApp.windows.map(ObjectIdentifier.init))
        let token = PendingTab(source: source, existing: existing)
        pending = token
        token.begin { openWindow(value: id) }
    }

    private static func clear(_ token: PendingTab) {
        if pending === token { pending = nil }
    }

    private final class PendingTab {
        let source: NSWindow
        let existing: Set<ObjectIdentifier>
        private var observer: NSObjectProtocol?
        private var done = false

        init(source: NSWindow, existing: Set<ObjectIdentifier>) {
            self.source = source
            self.existing = existing
        }

        func begin(open: () -> Void) {
            observer = NotificationCenter.default.addObserver(
                forName: NSWindow.didBecomeKeyNotification,
                object: nil,
                queue: .main
            ) { [weak self] note in
                self?.windowBecameKey(note)
            }
            open()
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
                self?.finish()
            }
        }

        private func windowBecameKey(_ note: Notification) {
            guard !done,
                  let window = note.object as? NSWindow,
                  window !== source,
                  !existing.contains(ObjectIdentifier(window)) else {
                return
            }
            done = true
            window.tabbingMode = .preferred
            window.tabbingIdentifier = source.tabbingIdentifier
            source.addTabbedWindow(window, ordered: .above)
            finish()
        }

        private func finish() {
            if let observer {
                NotificationCenter.default.removeObserver(observer)
                self.observer = nil
            }
            WindowTabber.clear(self)
        }
    }
}
