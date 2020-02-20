package io.github.pirocks

val a = listOf(1, 3, 4)
val b = listOf(1, null, 4)

fun <T : Any> List<T?>.convert(): List<T>? {
    if (this.any { it == null }) {
        return null
    } else {
        return this.filterNotNull()
    }
}