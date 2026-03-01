package com.mkihr_ojisan.minecraftd_bridge.fabric

import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents

private const val HISTORY_SIZE = 100

class Tps {
    private val tickTimeHistory = ArrayDeque<Long>(HISTORY_SIZE)
    private val msptHistory = ArrayDeque<Long>(HISTORY_SIZE)

    private var lastTickStartTime = 0L

    init {
        ServerTickEvents.START_SERVER_TICK.register { handleTickStart() }
        ServerTickEvents.END_SERVER_TICK.register { handleTickEnd() }
    }

    private fun handleTickStart() {
        lastTickStartTime = System.nanoTime()
    }

    private fun handleTickEnd() {
        val now = System.nanoTime()
        if (lastTickStartTime != 0L) {
            tickTimeHistory.addLast(now)
            msptHistory.addLast(now - lastTickStartTime)
            if (tickTimeHistory.size > HISTORY_SIZE) {
                tickTimeHistory.removeFirst()
            }
            if (msptHistory.size > HISTORY_SIZE) {
                msptHistory.removeFirst()
            }
        }
    }

    fun getTps(): Double? {
        if (tickTimeHistory.size < 2) {
            return null
        }
        val first = tickTimeHistory.first()
        val last = tickTimeHistory.last()
        val elapsedSeconds = (last - first) / 1_000_000_000.0
        val tickCount = tickTimeHistory.size - 1
        return tickCount / elapsedSeconds
    }

    fun getMspt(): Double? {
        if (msptHistory.isEmpty()) {
            return null
        }
        return msptHistory.average() / 1_000_000.0
    }
}