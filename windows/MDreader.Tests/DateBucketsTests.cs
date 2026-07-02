using System;
using MDreader.Util;
using Xunit;

namespace MDreader.Tests;

public class DateBucketsTests
{
    private static long NowAt1300()
    {
        var today = DateTime.Today; // local midnight
        var dt = today.AddHours(13);
        return new DateTimeOffset(dt).ToUnixTimeMilliseconds();
    }

    [Fact]
    public void SameDayIsToday()
    {
        var now = NowAt1300();
        Assert.Equal(DayBucket.Today, DateBuckets.Bucket(now, now));
        // 00:05 today is still today
        var early = DateTime.Today.AddMinutes(5);
        var earlyMillis = new DateTimeOffset(early).ToUnixTimeMilliseconds();
        Assert.Equal(DayBucket.Today, DateBuckets.Bucket(earlyMillis, now));
    }

    [Fact]
    public void PreviousDayIsYesterday()
    {
        var now = NowAt1300();
        Assert.Equal(DayBucket.Yesterday, DateBuckets.Bucket(now - DateBuckets.DayMillis, now));
    }

    [Fact]
    public void ThreeDaysAgoIsEarlier()
    {
        var now = NowAt1300();
        Assert.Equal(DayBucket.Earlier, DateBuckets.Bucket(now - 3 * DateBuckets.DayMillis, now));
    }

    [Fact]
    public void FormatProducesNonEmptyStringWithSlash()
    {
        var now = NowAt1300();
        var s = DateBuckets.Format(now - (9 * 3_600_000L + 30 * 60_000L));
        Assert.False(string.IsNullOrEmpty(s));
        Assert.Contains("/", s);
    }
}
