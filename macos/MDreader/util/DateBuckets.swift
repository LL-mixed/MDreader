import Foundation

enum DayBucket: CaseIterable {
    case today, yesterday, earlier

    var title: String {
        switch self {
        case .today: return "今天"
        case .yesterday: return "昨天"
        case .earlier: return "更早"
        }
    }
}

enum DateBuckets {
    private static let dayInterval: TimeInterval = 24 * 60 * 60

    static func bucket(_ date: Date, now: Date = .now, calendar: Calendar = .current) -> DayBucket {
        let todayStart = calendar.startOfDay(for: now)
        let itemStart = calendar.startOfDay(for: date)
        if itemStart >= todayStart { return .today }
        if itemStart >= todayStart.addingTimeInterval(-dayInterval) { return .yesterday }
        return .earlier
    }

    static func format(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy/MM/dd HH:mm"
        return formatter.string(from: date)
    }
}
