plugins {
    java
    kotlin("jvm")
}

repositories {
    mavenCentral()
}

group = "com.mkihr_ojisan.minecraftd_bridge"
version = "1.0.0"

base.archivesName = "minecraftd-bridge${project.path.replace(':', '-')}"