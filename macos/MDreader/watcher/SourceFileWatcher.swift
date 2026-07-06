import Foundation
import Darwin

/// Watches a single file path for content changes and reports them via `onChange`.
///
/// Built on `DispatchSource` (kqueue under the hood). Handles the common macOS
/// editor save styles:
/// - In-place write / `FileManager.replaceItem` (preserves inode): `.write`/`.extend`
///   fires directly.
/// - Atomic "delete old + write new" (`unlink` then `create`, or save-then-swap):
///   the original `fd` goes invalid (`.delete`/`.rename`); we attempt to reopen the
///   same path a few times and rebuild the source. If the path is gone for good
///   (file deleted/moved away), the watcher cancels itself.
///
/// All callbacks fire on a private serial queue (`mdr.sourcewatcher`). Callers are
/// responsible for hopping to the main thread before touching UI state.
///
/// Thread-safety note: `cancel()` may be called from any thread (including the
/// watcher's own queue — see the lifecycle hazard in `deinit`). State that crosses
/// the queue boundary is guarded by an `os_unfair_lock`.
final class SourceFileWatcher {

    /// Invoked when the watched file's content appears to have changed. Fires on
    /// the watcher's private queue. Set this before the file is modified.
    var onChange: (() -> Void)?

    /// Invoked once when the source can no longer be observed (file deleted or
    /// moved away and not recreated). Fires on the watcher's private queue. After
    /// this the watcher is cancelled and inert.
    var onCancel: (() -> Void)?

    private let path: String
    private let queue = DispatchQueue(label: "mdr.sourcewatcher")
    private let debounceInterval: TimeInterval

    /// Guards mutable state (`source`, `fd`, `cancelled`) shared between the
    /// watcher's queue and any external thread calling `cancel()` / `deinit`.
    private let lock = NSLock()
    private var source: DispatchSourceFileSystemObject?
    /// Current watched fd; closed by the active source's cancel handler (which
    /// captures it by value, so it closes even after `self` is gone).
    private var fd: CInt = -1
    private var cancelled = false

    init(path: String, debounceInterval: TimeInterval = 0.25) {
        self.path = path
        self.debounceInterval = debounceInterval
        queue.async { self.start() }
    }

    deinit {
        // CRITICAL: do NOT block here. This deinit can fire ON the watcher's own
        // queue (the common path: ReaderModel tears the watcher down by calling
        // cancel() — which enqueues an async block capturing self — then releases
        // its last reference; that async block runs on the queue and, on return,
        // releases self right here, on the queue's thread). A queue.sync from
        // that context deadlocks ("dispatch_sync called on queue already owned by
        // current thread") and libdispatch traps the process.
        //
        // DispatchSource.cancel() is itself thread-safe and non-blocking; the fd
        // is closed by the cancel handler (captured by value). No queue hop needed.
        lock.lock()
        let src = source
        lock.unlock()
        src?.cancel()
    }

    /// Public teardown; safe to call from any thread. Asynchronous: returns before
    /// the underlying `DispatchSource` is fully cancelled.
    func cancel() {
        queue.async { self.cancelLocked() }
    }

    /// Synchronous teardown meant for tests. Blocks until the watcher's
    /// `DispatchSource` is cancelled, so no live source outlives the caller —
    /// important in short-lived processes (e.g. the XCTest host) where an
    /// outstanding source can crash the process at shutdown.
    ///
    /// Only call from a thread OTHER than the watcher's own queue (deinit paths
    /// that land on the watcher queue must not use this).
    func cancelSync() {
        queue.sync { cancelLocked() }
    }

    // MARK: - Private (runs on `queue`)

    private func start() {
        guard !openDescriptor() else { return }
        // File doesn't exist right now (e.g. created later). Nothing to watch yet;
        // a future open path won't be observed, but that mirrors user intent: if
        // there is no source at watch time, there is nothing to track.
        cancelLocked()
    }

    @discardableResult
    private func openDescriptor() -> Bool {
        let newFd = Darwin.open(path, O_EVTONLY)
        guard newFd >= 0 else { return false }
        let src = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: newFd,
            eventMask: [.write, .delete, .rename, .extend, .link],
            queue: queue
        )
        src.setEventHandler { [weak self] in self?.handle(event: src) }
        // Capture `newFd` by value — must NOT capture self, so the fd is closed
        // even if self is deallocated before the cancel handler runs.
        src.setCancelHandler { close(newFd) }
        lock.lock()
        fd = newFd
        source = src
        lock.unlock()
        src.resume()
        return true
    }

    private func handle(event: DispatchSourceFileSystemObject) {
        let mask: DispatchSource.FileSystemEvent = event.mask
        let deleted = mask.contains(.delete)
        let renamed = mask.contains(.rename)
        let wrote = mask.contains(.write) || mask.contains(.extend)

        if deleted || renamed {
            // Atomic save or removal. Give the editor a moment to recreate the path,
            // then rebuild the source against the (possibly new) inode.
            reopen()
        } else if wrote {
            scheduleFire()
        }
    }

    /// Try to re-open the same path, retrying briefly to cover editors that
    /// `unlink` then immediately `create`. If it never comes back, give up.
    private func reopen() {
        let attempts = 3
        let stepUs: useconds_t = 50_000 // 50ms
        var reopened = false
        for _ in 0..<attempts {
            teardownSource() // release the stale fd/source first
            if openDescriptor() {
                reopened = true
                break
            }
            usleep(stepUs)
        }
        if reopened {
            scheduleFire()
        } else {
            cancelLocked()
        }
    }

    private var debounce: DispatchWorkItem?
    private func scheduleFire() {
        lock.lock(); let isCancelled = cancelled; lock.unlock()
        guard !isCancelled else { return }
        debounce?.cancel()
        let work = DispatchWorkItem { [weak self] in
            guard let self else { return }
            self.lock.lock()
            let isCancelled = self.cancelled
            self.lock.unlock()
            guard !isCancelled else { return }
            self.onChange?()
        }
        debounce = work
        queue.asyncAfter(deadline: .now() + debounceInterval, execute: work)
    }

    /// Cancels and releases the active source; its cancel handler (which captures
    /// the fd by value) closes the fd asynchronously on this same serial queue.
    private func teardownSource() {
        lock.lock()
        source?.cancel()
        source = nil
        fd = -1
        lock.unlock()
    }

    private func cancelLocked() {
        lock.lock()
        if cancelled {
            lock.unlock()
            return
        }
        cancelled = true
        lock.unlock()
        debounce?.cancel()
        debounce = nil
        teardownSource()
        onCancel?()
    }
}
