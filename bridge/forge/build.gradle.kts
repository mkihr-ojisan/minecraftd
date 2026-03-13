import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("common-conventions")
    id("net.minecraftforge.gradle") version "[7.0.3,8)"
    id("net.minecraftforge.renamer") version "1.0.6"
}

val minecraftVersion = sc.current.version
val forgeVersion = forgeVersion(minecraftVersion)
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

minecraft {
    mappings("official", sc.current.version)
}

repositories {
    minecraft.mavenizer(this)
    maven(fg.forgeMaven)
    maven(fg.minecraftLibsMaven)
}

dependencies {
    implementation(minecraft.dependency("net.minecraftforge:forge:${minecraftVersion}-${forgeVersion}"))
}

if (sc.current.parsed >= "1.20.6") {
    tasks.shadowJar {
        destinationDirectory = file("$rootDir/dist")
        archiveFileName.set("minecraftd-bridge-forge-${minecraftVersion}.jar")
    }
} else {
    renamer.classes(tasks.shadowJar) {
        map.from(minecraft.dependency.toSrgFile)
        output = file("$rootDir/dist/minecraftd-bridge-forge-${minecraftVersion}.jar")
    }
}