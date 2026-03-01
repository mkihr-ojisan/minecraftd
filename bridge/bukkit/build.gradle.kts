import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("common-conventions")
    id("io.typst.spigradle.spigot") version "4.0.0"
}

val minecraftVersion = sc.current.version
val javaVersion = javaVersion(minecraftVersion)
val kotlinJvmTarget = jvmTarget(minecraftVersion)

java {
    sourceCompatibility = javaVersion
    targetCompatibility = javaVersion
}

tasks.withType<KotlinCompile>().configureEach {
    compilerOptions {
        jvmTarget.set(kotlinJvmTarget)
    }
}

repositories {
    spigotRepos {
        papermc()
    }
}

dependencies {
    implementation("com.destroystokyo.paper:paper-api:1.16.5-R0.1-20211218.082619-371")
}

spigot {
    name = "minecraftd-bridge"
    apiVersion = minecraftVersion
    main = "com.mkihr_ojisan.minecraftd_bridge.bukkit.Plugin"
}