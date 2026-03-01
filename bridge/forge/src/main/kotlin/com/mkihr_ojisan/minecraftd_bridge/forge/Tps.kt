package com.mkihr_ojisan.minecraftd_bridge.forge

import net.minecraftforge.common.MinecraftForge
import net.minecraftforge.event.TickEvent

//? if >=1.21.11 {
 import net.minecraftforge.eventbus.api.listener.SubscribeEvent
//?} else
//import net.minecraftforge.eventbus.api.SubscribeEvent

const val HISTORY_SIZE = 100

class Tps {
    private val tickTimeHistory = ArrayDeque<Long>(HISTORY_SIZE)
    private val msptHistory = ArrayDeque<Long>(HISTORY_SIZE)

    private var lastTickStartTime = 0L

    init {
        MinecraftForge.EVENT_BUS.register(this)
    }

    //? if >=1.20.6 {
    @SubscribeEvent
    fun onServerTickStart(event: TickEvent.ServerTickEvent.Pre) {
        handleTickStart()
    }

    @SubscribeEvent
    fun onServerTickEnd(event: TickEvent.ServerTickEvent.Post) {
        handleTickEnd()
    }

    //? } else {
    /*@SubscribeEvent
    fun onServerTick(event: TickEvent.ServerTickEvent) {
        when (event.phase) {
            TickEvent.Phase.START -> handleTickStart()
            TickEvent.Phase.END -> handleTickEnd()
        }
    }
    *///? }

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