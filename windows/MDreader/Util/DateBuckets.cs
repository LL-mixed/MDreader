namespace MDreader.Util;

/// <summary>
/// Day bucket: 今天 / 昨天 / 更早. Port of linux date_buckets.rs / macOS DateBuckets.swift.
/// </summary>
public enum DayBucket { Today, Yesterday, Earlier }

public static class DateBuckets
{
    public const long DayMillis = 86_400_000L;

    public static readonly DayBucket[] All =
    {
        DayBucket.Today, DayBucket.Yesterday, DayBucket.Earlier,
    };

    public static string Title(this DayBucket b) => b switch
    {
        DayBucket.Today => "今天",
        DayBucket.Yesterday => "昨天",
        _ => "更早",
    };

    public static DayBucket Bucket(long dateMillis, long nowMillis)
    {
        var todayStart = StartOfDayLocal(nowMillis);
        var itemStart = StartOfDayLocal(dateMillis);
        if (itemStart >= todayStart) return DayBucket.Today;
        if (itemStart >= todayStart - DayMillis) return DayBucket.Yesterday;
        return DayBucket.Earlier;
    }

    /// <summary>Formats as "yyyy/MM/dd HH:mm" in the local time zone.</summary>
    public static string Format(long millis)
    {
        var dt = DateTimeOffset.FromUnixTimeMilliseconds(millis).LocalDateTime;
        return dt.ToString("yyyy/MM/dd HH:mm");
    }

    private static long StartOfDayLocal(long millis)
    {
        var local = DateTimeOffset.FromUnixTimeMilliseconds(millis).LocalDateTime;
        var midnight = local.Date;
        return new DateTimeOffset(midnight).ToUnixTimeMilliseconds();
    }
}
