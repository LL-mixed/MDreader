import Foundation

struct OutlineItem: Identifiable, Equatable, Codable {
    let index: Int
    let level: Int
    let text: String

    var id: Int { index }
}
