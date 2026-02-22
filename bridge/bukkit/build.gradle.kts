plugins {
    id("io.typst.spigradle.spigot") version "4.0.0"
}

repositories {
    spigotRepos {
        spigotmc()
    }
}

dependencies {
    compileOnly(spigots.spigot.api)
}

spigot {
    name = "minecraftd-bridge"
    apiVersion = "1.7"
}