plugins {
    id("io.typst.spigradle.spigot") version "4.0.0"
}

repositories {
    spigotRepos {
        papermc()
    }
}

dependencies {
    compileOnly("com.destroystokyo.paper:paper-api:1.16.5-R0.1-20211218.082619-371")
}

spigot {
    name = "minecraftd-bridge"
    apiVersion = "1.7"
}