import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("common-conventions")
    id("net.fabricmc.fabric-loom-remap") version "1.15-SNAPSHOT"
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

dependencies {
    minecraft("com.mojang:minecraft:${minecraftVersion}")
    mappings(loom.officialMojangMappings())
    modImplementation("net.fabricmc:fabric-loader:0.18.4")
    modImplementation("net.fabricmc.fabric-api:fabric-api:${fabricApiVersion(minecraftVersion)}")
}

tasks.processResources {
    val modVersion = project.version
    inputs.property("version", modVersion)
    filesMatching("fabric.mod.json") {
        expand("version" to modVersion)
    }
}

tasks.remapJar {
    inputFile = tasks.shadowJar.flatMap { it.archiveFile }
    archiveClassifier.set("remapped")
}
