# MDreader ProGuard rules

# Keep Room generated/DAO implementations
-keep class com.mdreader.data.** { *; }

# Kotlin metadata
-keepattributes *Annotation*
-keepclassmembers class * {
    @androidx.room.* <methods>;
}
