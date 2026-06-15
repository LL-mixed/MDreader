package com.mdreader.util

import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.Calendar

class DateBucketsTest {

    private val dayMs = 24L * 60 * 60 * 1000

    /** Today at the given hour, in the machine's default timezone. */
    private fun todayAt(hour: Int, minute: Int = 0): Long {
        val cal = Calendar.getInstance()
        cal.set(Calendar.HOUR_OF_DAY, hour)
        cal.set(Calendar.MINUTE, minute)
        cal.set(Calendar.SECOND, 0)
        cal.set(Calendar.MILLISECOND, 0)
        return cal.timeInMillis
    }

    @Test
    fun sameDayIsToday() {
        val now = todayAt(13)
        assertEquals(DayBucket.TODAY, DateBuckets.bucket(now, now))
        assertEquals(DayBucket.TODAY, DateBuckets.bucket(todayAt(0, 5), now))
    }

    @Test
    fun previousDayIsYesterday() {
        val now = todayAt(13)
        assertEquals(DayBucket.YESTERDAY, DateBuckets.bucket(now - dayMs, now))
    }

    @Test
    fun threeDaysAgoIsEarlier() {
        val now = todayAt(13)
        assertEquals(DayBucket.EARLIER, DateBuckets.bucket(now - 3 * dayMs, now))
    }

    @Test
    fun formatProducesNonEmptyString() {
        val formatted = DateBuckets.format(todayAt(9, 30))
        assert(formatted.isNotBlank())
        assert(formatted.contains("/"))
    }
}
