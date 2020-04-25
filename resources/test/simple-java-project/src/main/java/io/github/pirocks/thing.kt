package io.github.pirocks

import sun.jvmstat.monitor.IntegerMonitor

val a = listOf(1, 3, 4)
val b = listOf(1, null, 4)

fun <T : Any> List<T?>.convert(): List<T>? {
    val test = mutableListOf<MutableList<Int>>()
    test.map { it.map { it != 0 }.toMutableList() }.toMutableList()
    if (this.any { it == null }) {
        return null
    } else {
        return this.filterNotNull()
    }

}