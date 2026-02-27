package com.mkihr_ojisan.minecraftd_bridge.bukkit

import com.mkihr_ojisan.minecraftd_bridge.common.Api
import org.bukkit.Bukkit

open class BukkitApi(private val plugin: Plugin) : Api {
    private val server = Bukkit.getServer()
    private val scheduler = server.scheduler
    private val tps = Tps(plugin)
    private val mstp = if (Mspt.isSupported()) Mspt(plugin) else null

    override fun <T> runOnMainThread(task: () -> T): T {
        return scheduler.callSyncMethod(plugin, task).get()
    }

    override fun logError(message: String) {
        plugin.logger.severe(message)
    }

    override fun logInfo(message: String) {
        plugin.logger.info(message)
    }

    override fun getTPS(): Double? {
        return tps.getTps()
    }

    override fun getMSPT(): Double? {
        return mstp?.getMspt()
    }

    override fun getPlayerCount(): Int? {
        return server.onlinePlayers.size
    }

    override fun getEntityCount(): Int? {
        var count = 0
        for (world in server.worlds) {
            count += world.entities.size
        }
        return count
    }

    override fun getLoadedChunkCount(): Int? {
        var count = 0
        for (world in server.worlds) {
            count += world.loadedChunks.size
        }
        return count
    }
}