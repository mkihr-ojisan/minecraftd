import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    java
    kotlin("jvm") version "2.3.10" apply false
    id("com.gradleup.shadow") version "9.3.1" apply false
}

allprojects {
    group = "com.mkihr_ojisan.minecraftd_bridge"
    version = "1.0.0"

    repositories {
        mavenCentral()
    }
}

subprojects {
    apply(plugin = "java")
    apply(plugin = "org.jetbrains.kotlin.jvm")

    java {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    tasks.withType<KotlinCompile>().configureEach {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_1_8)
        }
    }

    dependencies {
        implementation(kotlin("stdlib"))
    }

    if (name != "common") {
        pluginManager.apply("com.gradleup.shadow")

        dependencies {
            implementation(project(":common"))
        }

        tasks.named<com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar>("shadowJar") {
            // disable "-all" postfix in the generated JAR name
            archiveClassifier.set("")
        }

        tasks.named("assemble") {
            dependsOn("shadowJar")
        }
    }
}
