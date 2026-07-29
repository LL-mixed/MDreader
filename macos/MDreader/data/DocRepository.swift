import Foundation
import OSLog
import SwiftData

final class DocRepository {
    static let defaultTitle = "未命名文档"
    static let recoveredTitlePrefix = "恢复的文档"

    private static let logger = Logger(
        subsystem: "com.mdreader.macos",
        category: "library"
    )

    let container: ModelContainer
    let docsDir: URL

    static var defaultLibraryDirectory: URL {
        let appSupport = FileManager.default.urls(
            for: .applicationSupportDirectory,
            in: .userDomainMask
        ).first!
        return appSupport.appendingPathComponent("MDreader", isDirectory: true)
    }

    static func makePersistentContainer(
        libraryDirectory: URL = defaultLibraryDirectory
    ) throws -> ModelContainer {
        try FileManager.default.createDirectory(
            at: libraryDirectory,
            withIntermediateDirectories: true
        )
        let storeURL = libraryDirectory.appendingPathComponent("library.store")
        let configuration = ModelConfiguration(url: storeURL)
        return try ModelContainer(
            for: CachedDoc.self,
            configurations: configuration
        )
    }

    init(container: ModelContainer, docsDir: URL? = nil) {
        self.container = container
        if let docsDir {
            self.docsDir = docsDir
        } else {
            self.docsDir = Self.defaultLibraryDirectory
                .appendingPathComponent("docs", isDirectory: true)
        }
        recoverOrphanedCacheFiles()
    }

    @discardableResult
    func cache(title: String, markdown: String, sourceURI: String?) -> UUID {
        let hash = ContentHash.sha256Hex(markdown)
        let now = Date()
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(predicate: #Predicate { $0.contentHash == hash })
        if let existing = (try? context.fetch(descriptor))?.first {
            existing.openedAt = now
            do {
                try context.save()
            } catch {
                Self.logger.error(
                    "Failed to update library metadata: \(error.localizedDescription, privacy: .public)"
                )
            }
            return existing.id
        }
        let resolvedTitle = title.isEmpty ? Self.defaultTitle : title
        let doc = CachedDoc(
            title: resolvedTitle,
            contentHash: hash,
            sourceURI: sourceURI,
            charCount: markdown.count,
            sizeBytes: markdown.utf8.count,
            cachedAt: now,
            openedAt: now
        )
        context.insert(doc)
        do {
            try context.save()
        } catch {
            Self.logger.error(
                "Failed to save library metadata: \(error.localizedDescription, privacy: .public)"
            )
        }
        DocStore.write(docsDir: docsDir, id: doc.id, markdown: markdown)
        return doc.id
    }

    func all() -> [DocInfo] {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(sortBy: [SortDescriptor(\.openedAt, order: .reverse)])
        let docs: [CachedDoc]
        do {
            docs = try context.fetch(descriptor)
        } catch {
            Self.logger.error(
                "Failed to load library metadata: \(error.localizedDescription, privacy: .public)"
            )
            return []
        }
        return docs.map {
            DocInfo(id: $0.id, title: $0.title, contentHash: $0.contentHash, sourceURI: $0.sourceURI, openedAt: $0.openedAt, favorite: $0.favorite, charCount: $0.charCount)
        }
    }

    @discardableResult
    func recoverOrphanedCacheFiles() -> Int {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>()
        let existingDocs: [CachedDoc]
        do {
            existingDocs = try context.fetch(descriptor)
        } catch {
            Self.logger.error(
                "Failed to inspect library metadata: \(error.localizedDescription, privacy: .public)"
            )
            return 0
        }

        var knownIDs = Set(existingDocs.map(\.id))
        var knownHashes = Set(existingDocs.map(\.contentHash))
        guard let cachedFiles = try? FileManager.default.contentsOfDirectory(
            at: docsDir,
            includingPropertiesForKeys: [.contentModificationDateKey],
            options: [.skipsHiddenFiles]
        ) else {
            return 0
        }

        var recoveredCount = 0
        for fileURL in cachedFiles.sorted(by: { $0.lastPathComponent < $1.lastPathComponent }) {
            guard fileURL.pathExtension.lowercased() == "md",
                  let id = UUID(uuidString: fileURL.deletingPathExtension().lastPathComponent),
                  !knownIDs.contains(id),
                  let markdown = try? String(contentsOf: fileURL, encoding: .utf8) else {
                continue
            }

            let hash = ContentHash.sha256Hex(markdown)
            guard !knownHashes.contains(hash) else { continue }

            let values = try? fileURL.resourceValues(
                forKeys: [.contentModificationDateKey]
            )
            let cachedAt = values?.contentModificationDate ?? .now
            let doc = CachedDoc(
                id: id,
                title: Self.recoveredTitle(from: markdown, id: id),
                contentHash: hash,
                sourceURI: nil,
                charCount: markdown.count,
                sizeBytes: markdown.utf8.count,
                cachedAt: cachedAt,
                openedAt: cachedAt
            )
            context.insert(doc)
            knownIDs.insert(id)
            knownHashes.insert(hash)
            recoveredCount += 1
        }

        guard recoveredCount > 0 else { return 0 }
        do {
            try context.save()
            Self.logger.notice("Recovered \(recoveredCount) library entries from cached files")
            return recoveredCount
        } catch {
            Self.logger.error(
                "Failed to recover library metadata: \(error.localizedDescription, privacy: .public)"
            )
            return 0
        }
    }

    private static func recoveredTitle(from markdown: String, id: UUID) -> String {
        for line in markdown.split(separator: "\n", omittingEmptySubsequences: false) {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            guard trimmed.hasPrefix("# ") else { continue }
            let title = trimmed.dropFirst(2).trimmingCharacters(in: .whitespaces)
            if !title.isEmpty { return title }
        }
        return "\(recoveredTitlePrefix) \(id.uuidString.prefix(8))"
    }

    func search(_ query: String) -> [DocInfo] {
        let q = query.lowercased()
        return all().filter { $0.title.lowercased().contains(q) }
    }

    func loadContent(id: UUID) -> String? {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(predicate: #Predicate { $0.id == id })
        if let doc = (try? context.fetch(descriptor))?.first {
            doc.openedAt = Date()
            try? context.save()
        }
        return DocStore.read(docsDir: docsDir, id: id)
    }

    func setFavorite(id: UUID, favorite: Bool) {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(predicate: #Predicate { $0.id == id })
        if let doc = (try? context.fetch(descriptor))?.first {
            doc.favorite = favorite
            try? context.save()
        }
    }

    func delete(id: UUID) {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(predicate: #Predicate { $0.id == id })
        if let doc = (try? context.fetch(descriptor))?.first {
            context.delete(doc)
            try? context.save()
        }
        DocStore.delete(docsDir: docsDir, id: id)
    }

    /// Re-reads the original file backing `id`. If it exists and its content differs
    /// from the cached snapshot, updates the cached content + metadata. Returns true
    /// when a refresh actually happened. No-op (false) when there is no source or the
    /// content is unchanged.
    @discardableResult
    func refreshFromSource(id: UUID) -> Bool {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(predicate: #Predicate { $0.id == id })
        guard let doc = (try? context.fetch(descriptor))?.first,
              let path = doc.sourceURI,
              let text = try? String(contentsOf: URL(fileURLWithPath: path), encoding: .utf8) else {
            return false
        }
        let hash = ContentHash.sha256Hex(text)
        guard hash != doc.contentHash else { return false }
        doc.contentHash = hash
        doc.charCount = text.count
        doc.sizeBytes = text.utf8.count
        doc.openedAt = Date()
        try? context.save()
        DocStore.write(docsDir: docsDir, id: id, markdown: text)
        return true
    }
}
