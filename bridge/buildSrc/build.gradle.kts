plugins {
    `kotlin-dsl`
}

repositories {
    gradlePluginPortal()
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-gradle-plugin:2.3.10")
    implementation("com.gradleup.shadow:com.gradleup.shadow.gradle.plugin:9.3.2")
}