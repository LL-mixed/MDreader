import XCTest
import Foundation
@testable import MDreader

/// Data-driven tests driven by the shared spec files under shared/specs. The
/// input→output contract is defined ONCE in those JSON files and consumed by
/// Android, macOS, and Linux alike — so a spec edit propagates to all platforms.
final class SharedSpecTests: XCTestCase {

    /// Loads cases from `shared/specs/<name>.json`, resolved relative to the
    /// macos/ test dir (the repo root is the parent of macos/).
    private func cases(_ name: String) -> [[String: Any]] {
        // macos/Tests/ → macos/ → repo root → shared/specs
        let here = URL(fileURLWithPath: #file).deletingLastPathComponent()
        let repoRoot = here.deletingLastPathComponent().deletingLastPathComponent()
        let url = repoRoot.appendingPathComponent("shared/specs/\(name).json")
        let data = try! Data(contentsOf: url)
        let json = try! JSONSerialization.jsonObject(with: data) as! [String: Any]
        return json["cases"] as! [[String: Any]]
    }

    func testContentHashMatchesSpec() {
        for c in cases("content_hash") {
            let input = c["input"] as! String
            let expected = c["expected"] as! String
            XCTAssertEqual(expected, ContentHash.sha256Hex(input), "sha256(\(input))")
        }
    }

    func testTitlesMatchesSpec() {
        for c in cases("titles") {
            let input = c["input"] as! String
            let expected = c["expected"] as! String
            XCTAssertEqual(expected, Titles.fromPath(input), "Titles.fromPath(\(input))")
        }
    }

    func testMermaidFenceMatchesSpec() {
        for c in cases("mermaid_fence") {
            let name = (c["name"] as? String) ?? "?"
            let input = c["input"] as! String
            let expected = c["expected"] as! String
            XCTAssertEqual(expected, MermaidFenceNormalizer.normalize(input), "mermaid case '\(name)'")
        }
    }
}
