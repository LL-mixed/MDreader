import Foundation

struct DocInfo: Identifiable, Equatable {
    let id: UUID
    let title: String
    let contentHash: String
    let sourceURI: String?
    let openedAt: Date
    let favorite: Bool
    let charCount: Int
}
