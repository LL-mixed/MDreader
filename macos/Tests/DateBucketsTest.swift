import XCTest
@testable import MDreader

final class DateBucketsTest: XCTestCase {
    private let dayInterval: TimeInterval = 24 * 60 * 60

    private func date(hour: Int, minute: Int = 0, calendar: Calendar = .current) -> Date {
        var components = calendar.dateComponents([.year, .month, .day], from: Date())
        components.hour = hour
        components.minute = minute
        components.second = 0
        return calendar.date(from: components)!
    }

    func testSameDayIsToday() {
        let now = date(hour: 13)
        XCTAssertEqual(DateBuckets.bucket(now, now: now), .today)
        XCTAssertEqual(DateBuckets.bucket(date(hour: 0, minute: 5), now: now), .today)
    }

    func testPreviousDayIsYesterday() {
        let now = date(hour: 13)
        XCTAssertEqual(DateBuckets.bucket(now.addingTimeInterval(-dayInterval), now: now), .yesterday)
    }

    func testThreeDaysAgoIsEarlier() {
        let now = date(hour: 13)
        XCTAssertEqual(DateBuckets.bucket(now.addingTimeInterval(-3 * dayInterval), now: now), .earlier)
    }

    func testFormatProducesNonEmptyStringWithSlash() {
        let formatted = DateBuckets.format(date(hour: 9, minute: 30))
        XCTAssertFalse(formatted.isEmpty)
        XCTAssertTrue(formatted.contains("/"))
    }
}
