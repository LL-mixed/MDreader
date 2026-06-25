import Foundation

struct BuildInfo {
    let gitHash: String
    let buildTime: String
    let author: String

    static let current: BuildInfo = {
        guard let url = Bundle.main.url(forResource: "build-info", withExtension: "json"),
              let data = try? Data(contentsOf: url),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return BuildInfo(gitHash: "dev", buildTime: "dev", author: "MDreader")
        }
        return BuildInfo(
            gitHash: obj["gitHash"] ?? "dev",
            buildTime: obj["buildTime"] ?? "dev",
            author: obj["author"] ?? "MDreader"
        )
    }()
}
