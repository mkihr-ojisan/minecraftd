package com.mkihr_ojisan.minecraftd_bridge.bukkit

import org.bukkit.event.Listener
import com.destroystokyo.paper.event.server.ServerTickEndEvent
import org.bukkit.Bukkit
import org.bukkit.event.EventHandler

private const val HISTORY_SIZE = 100

class Mspt(plugin: Plugin) : Listener {
    companion object {
        fun isSupported(): Boolean {
            return try {
                Class.forName("com.destroystokyo.paper.event.server.ServerTickEndEvent")
                true
            } catch (_: ClassNotFoundException) {
                false
            }
        }
    }

    init {
        Bukkit.getServer().pluginManager.registerEvents(this, plugin)
    }

    private val history = ArrayDeque<Double>(HISTORY_SIZE)

    @EventHandler
    private fun onServerTickEnd(event: ServerTickEndEvent) {
        if (history.size == HISTORY_SIZE) {
            history.removeFirst()
        }
        history.addLast(event.tickDuration)
    }

    fun getMspt(): Double? {
        if (history.isEmpty()) {
            return null
        }
        return history.average()
    }
}