package com.mkihr_ojisan.minecraftd_bridge.forge

import com.mkihr_ojisan.minecraftd_bridge.common.RequestHandler
import com.mkihr_ojisan.minecraftd_bridge.common.SocketServer
import net.minecraftforge.common.MinecraftForge
import net.minecraftforge.event.server.ServerStartedEvent
import net.minecraftforge.event.server.ServerStoppingEvent
import net.minecraftforge.fml.common.Mod
import net.minecraftforge.fml.javafmlmod.FMLJavaModLoadingContext
import org.apache.logging.log4j.LogManager

//? if >=1.21.11 {
 import net.minecraftforge.eventbus.api.listener.SubscribeEvent
//?} else
//import net.minecraftforge.eventbus.api.SubscribeEvent

@Suppress("unused")
@Mod(BridgeMod.MOD_ID)
class BridgeMod(context: FMLJavaModLoadingContext) {
    companion object {
        const val MOD_ID = "minecraftd_bridge"
        val LOGGER = LogManager.getLogger("minecraftd_bridge")
    }

    private var socketServer: SocketServer? = null

    init {
        MinecraftForge.EVENT_BUS.register(this)
    }

    @SubscribeEvent
    fun onServerStarted(event: ServerStartedEvent) {
        val server = event.server
        val api = ForgeApi(LOGGER, server)
        val requestHandler = RequestHandler(api)
        val socketServer = SocketServer(api)
        this.socketServer = socketServer
        socketServer.start(requestHandler)
    }

    @SubscribeEvent
    fun onServerStopping(event: ServerStoppingEvent) {
        socketServer?.close()
    }
}