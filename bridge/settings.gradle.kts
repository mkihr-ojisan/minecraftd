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
include("common", "bukkit")