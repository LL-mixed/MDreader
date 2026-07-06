import XCTest
@testable import MDreader

/// Unit tests for `SourceFileWatcher` in isolation. These are async because the
/// watcher dispatches callbacks on a private queue.
///
/// Each test tears the watcher down synchronously (`cancelSync()`) before it
/// returns, so no live `DispatchSource` outlives the test — an outstanding source
/// at process-shutdown crashes the XCTest host.
final class SourceFileWatcherTests: XCTestCase {

    private func writeSource(_ name: String, _ body: String) throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let url = dir.appendingPathComponent(name)
        try body.write(to: url, atomically: true, encoding: .utf8)
        return url
    }

    func testFiresOnChangeWhenFileIsWritten() async throws {
        let url = try writeSource("note.md", "# v1")
        defer { try? FileManager.default.removeItem(at: url.deletingLastPathComponent()) }

        let fired = expectation(description: "onChange fired")
        let watcher = SourceFileWatcher(path: url.path)
        watcher.onChange = { fired.fulfill() }

        // Give the watcher a moment to arm its DispatchSource.
        try await Task.sleep(nanoseconds: 200_000_000)
        try "# v2".write(to: url, atomically: true, encoding: .utf8)

        await fulfillment(of: [fired], timeout: 3.0)
        watcher.cancelSync()
    }

    func testSurvivesAtomicReplaceByPath() async throws {
        // Simulate editors that save via unlink+create (new inode at same path).
        let url = try writeSource("note.md", "# v1")
        defer { try? FileManager.default.removeItem(at: url.deletingLastPathComponent()) }

        let fired = expectation(description: "onChange fired after recreate")
        let watcher = SourceFileWatcher(path: url.path)
        watcher.onChange = { fired.fulfill() }

        try await Task.sleep(nanoseconds: 200_000_000)
        try FileManager.default.removeItem(at: url)
        try "# v2".write(to: url, atomically: true, encoding: .utf8)

        await fulfillment(of: [fired], timeout: 3.0)
        watcher.cancelSync()
    }

    func testCancelsWhenFileRemovedForGood() async throws {
        let url = try writeSource("note.md", "# v1")
        defer { try? FileManager.default.removeItem(at: url.deletingLastPathComponent()) }

        let cancelled = expectation(description: "onCancel fired")
        let watcher = SourceFileWatcher(path: url.path)
        watcher.onCancel = { cancelled.fulfill() }

        try await Task.sleep(nanoseconds: 200_000_000)
        try FileManager.default.removeItem(at: url)

        await fulfillment(of: [cancelled], timeout: 3.0)
        // The watcher already cancelled itself; this is a harmless no-op that
        // guarantees no live source remains.
        watcher.cancelSync()
    }

    /// Regression: tearing down a watcher must not deadlock/crash even when the
    /// watcher's last reference is released ON its own queue. This is the exact
    /// production path — `ReaderModel.teardownWatcher()` calls `cancel()` (which
    /// enqueues an async block capturing self) then drops its reference; the async
    /// block runs on the watcher queue and releases self there. A `queue.sync` in
    /// `deinit` would trap with "dispatch_sync called on queue already owned by
    /// current thread". This test hammers that path repeatedly; before the fix it
    /// crashed the test host within a few iterations.
    func testTeardownFromWatcherQueueDoesNotDeadlock() throws {
        for _ in 0..<50 {
            let url = try writeSource("note.md", "# x")
            // Capture the watcher only weakly from the queue; cancel() enqueues a
            // block that holds the last strong reference and runs on the queue.
            var watcher: SourceFileWatcher? = SourceFileWatcher(path: url.path)
            watcher!.cancel()                // async; last ref released on the queue
            watcher = nil                    // drop our ref immediately
            // Give the queue a moment to drain (run the cancel block + deinit).
            Thread.sleep(forTimeInterval: 0.05)
            try? FileManager.default.removeItem(at: url.deletingLastPathComponent())
        }
        // Reaching here means no deadlock/trap across 50 teardown cycles.
        XCTAssertTrue(true)
    }
}
