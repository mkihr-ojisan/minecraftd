package com.mkihr_ojisan.minecraftd_bridge.common

interface Api {
    fun <T> runOnMainThread(task: () -> T): T
    fun log_error(message: String)
    fun log_info(message: String)
    fun getPlayerCount(): Int
}