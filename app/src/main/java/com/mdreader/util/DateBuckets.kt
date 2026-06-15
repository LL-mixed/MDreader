package com.mdreader.util

import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Locale

/** Coarse day buckets used to group the document library by recency. */
enum class DayBucket { TODAY, YESTERDAY, EARLIER }

/** Groups timestamps into recency buckets and formats them for display. */
object DateBuckets {

    private const val DAY_MS = 24L * 60 * 60 * 1000

    fun bucket(epochMillis: Long, nowMillis: Long): DayBucket {
        val todayStart = dayStart(nowMillis)
        val itemStart = dayStart(epochMillis)
        return when {
            itemStart >= todayStart -> DayBucket.TODAY
            itemStart >= todayStart - DAY_MS -> DayBucket.YESTERDAY
            else -> DayBucket.EARLIER
        }
    }

    fun format(epochMillis: Long): String =
        SimpleDateFormat("yyyy/MM/dd HH:mm", Locale.getDefault()).format(java.util.Date(epochMillis))

    private fun dayStart(millis: Long): Long {
        val cal = Calendar.getInstance()
        cal.timeInMillis = millis
        cal.set(Calendar.HOUR_OF_DAY, 0)
        cal.set(Calendar.MINUTE, 0)
        cal.set(Calendar.SECOND, 0)
        cal.set(Calendar.MILLISECOND, 0)
        return cal.timeInMillis
    }
}
