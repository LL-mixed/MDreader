import Foundation
import SwiftData

@Model
final class CachedDoc {
    @Attribute(.unique) var id: UUID
    var title: String
    var contentHash: String
    var sourceURI: String?
    var charCount: Int
    var sizeBytes: Int
    var cachedAt: Date
    var openedAt: Date
    var favorite: Bool

    init(
        id: UUID = UUID(),
        title: String,
        contentHash: String,
        sourceURI: String?,
        charCount: Int,
        sizeBytes: Int,
        cachedAt: Date = .now,
        openedAt: Date = .now,
        favorite: Bool = false
    ) {
        self.id = id
        self.title = title
        self.contentHash = contentHash
        self.sourceURI = sourceURI
        self.charCount = charCount
        self.sizeBytes = sizeBytes
        self.cachedAt = cachedAt
        self.openedAt = openedAt
        self.favorite = favorite
    }
}
