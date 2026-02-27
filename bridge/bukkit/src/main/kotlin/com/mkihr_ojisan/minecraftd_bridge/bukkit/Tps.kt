package com.mkihr_ojisan.minecraftd_bridge.bukkit

import org.bukkit.Bukkit

private const val HISTORY_SIZE = 100

class Tps(plugin: Plugin) {
    init {
        Bukkit.getScheduler().runTaskTimer(plugin, ::tick, 0L, 1L)
    }

    private val history = ArrayDeque<Long>(HISTORY_SIZE)

    private fun tick() {
        val now = System.nanoTime()
        if (history.size == HISTORY_SIZE) {
            history.removeFirst()
        }
        history.addLast(now)
    }

    fun getTps(): Double? {
        if (history.size < 2) {
            return null
        }
        val first = history.first()
        val last = history.last()
        val elapsedSeconds = (last - first) / 1_000_000_000.0
        val tickCount = history.size - 1
        return tickCount / elapsedSeconds
    }
}