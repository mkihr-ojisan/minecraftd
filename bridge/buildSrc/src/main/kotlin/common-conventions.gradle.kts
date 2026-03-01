import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar

plugins {
    id("base-conventions")
    id("com.gradleup.shadow")
}

dependencies {
    implementation(project(":common:all"))
}

tasks.named<ShadowJar>("shadowJar") {
    // disable "-all" postfix in the generated JAR name
    archiveClassifier.set("")
    // PaperMC uses an older version of protobuf, so we need to relocate our protobuf dependency to avoid conflicts
    relocate("com.google.protobuf", "com.mkihr_ojisan.minecraftd_bridge.protobuf")
}

tasks.named("assemble") {
    dependsOn("shadowJar")
}