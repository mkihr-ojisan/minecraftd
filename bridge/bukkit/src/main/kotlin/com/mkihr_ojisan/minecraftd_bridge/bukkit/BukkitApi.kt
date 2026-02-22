package com.mkihr_ojisan.minecraftd_bridge.bukkit

import com.mkihr_ojisan.minecraftd_bridge.common.Api
import org.bukkit.Bukkit

class BukkitApi(private val plugin: Plugin) : Api {
    override fun <T> runOnMainThread(task: () -> T): T {
        return Bukkit.getScheduler().callSyncMethod(plugin) {
            task()
        }.get()
    }

    override fun getPlayerCount(): Int {
        return Bukkit.getServer().onlinePlayers.size
    }
}