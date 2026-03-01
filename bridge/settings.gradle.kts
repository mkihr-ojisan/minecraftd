dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
    versionCatalogs {
        create("spigots") {
            from("io.typst:spigot-catalog:1.0.0")
        }
    }
}

rootProject.name = "minecraftd-bridge"

pluginManagement {
    repositories {
        gradlePluginPortal()
        maven("https://maven.kikugie.dev/snapshots") { name = "KikuGie Snapshots" }
        maven("https://maven.fabricmc.net/") { name = "Fabric" }
    }
}

plugins {
    id("dev.kikugie.stonecutter") version "0.8.3"
}

stonecutter {
    create(rootProject) {
        branch("common") {
            versions("all")
        }
        branch("bukkit") {
            versions(
                "1.7",
                "1.13",
            )
        }
        branch("forge") {
            versions(
                "1.18.2",
                "1.19.2",
                "1.20.1",
                "1.21.1",
                "1.21.11",
            )
        }
        branch("fabric") {
            versions(
                "1.16.5",
                "1.18.2",
                "1.19.2",
                "1.20.1",
                "1.21.1",
                "1.21.11",
            )
        }

        mapBuilds { _, _ -> "build.gradle.kts" }

        vcsVersion = "1.21.11"
    }
}