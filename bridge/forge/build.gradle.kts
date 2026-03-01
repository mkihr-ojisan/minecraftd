import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("common-conventions")
    id("net.minecraftforge.gradle") version "[7.0.3,8)"
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