import org.gradle.api.JavaVersion
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

private fun javaVersionNum(minecraftVersion: String): Int {
    val split = minecraftVersion.split(".")
    if (split.size < 2) {
        throw IllegalArgumentException("Invalid Minecraft version: $minecraftVersion")
    }

    val major = split[0].toIntOrNull() ?: throw IllegalArgumentException("Invalid Minecraft version: $minecraftVersion")
    val minor = split[1].toIntOrNull() ?: throw IllegalArgumentException("Invalid Minecraft version: $minecraftVersion")

    if (major != 1) {
        throw IllegalArgumentException("Unsupported Minecraft version: $minecraftVersion")
    }

    return when {
        minor <= 16 -> 8
        minor == 17 -> 16
        minor <= 20 -> 17
        else -> 21
    }
}

fun javaVersion(minecraftVersion: String): JavaVersion {
    return when (val versionNum = javaVersionNum(minecraftVersion)) {
        8 -> JavaVersion.VERSION_1_8
        16 -> JavaVersion.VERSION_16
        17 -> JavaVersion.VERSION_17
        21 -> JavaVersion.VERSION_21
        else -> error("unreachable")
    }
}

fun jvmTarget(minecraftVersion: String): JvmTarget {
    return when (javaVersionNum(minecraftVersion)) {
        8 -> JvmTarget.JVM_1_8
        16 -> JvmTarget.JVM_16
        17 -> JvmTarget.JVM_17
        21 -> JvmTarget.JVM_21
        else -> error("unreachable")
    }
}