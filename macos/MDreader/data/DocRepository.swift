import Foundation
import SwiftData

final class DocRepository {
    static let defaultTitle = "未命名文档"

    let container: ModelContainer
    let docsDir: URL

    init(container: ModelContainer, docsDir: URL? = nil) {
        self.container = container
        if let docsDir {
            self.docsDir = docsDir
        } else {
            let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            self.docsDir = appSupport
                .appendingPathComponent("MDreader", isDirectory: true)
                .appendingPathComponent("docs", isDirectory: true)
        }
    }

    func cache(title: String, markdown: String, sourceURI: String?) {
        let hash = ContentHash.sha256Hex(markdown)
        let now = Date()
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(predicate: #Predicate { $0.contentHash == hash })
        if let existing = (try? context.fetch(descriptor))?.first {
            existing.openedAt = now
            try? context.save()
            return
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
        try? context.save()
        DocStore.write(docsDir: docsDir, id: doc.id, markdown: markdown)
    }

    func all() -> [DocInfo] {
        let context = ModelContext(container)
        let descriptor = FetchDescriptor<CachedDoc>(sortBy: [SortDescriptor(\.openedAt, order: .reverse)])
        return ((try? context.fetch(descriptor)) ?? []).map {
            DocInfo(id: $0.id, title: $0.title, contentHash: $0.contentHash, openedAt: $0.openedAt, favorite: $0.favorite, charCount: $0.charCount)
        }
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
}
